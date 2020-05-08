use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{export::TokenStream2, *};

macro_rules! unwrap {
    ($input:expr, $arm:pat => $value:expr) => {{
        match $input {
            $arm => $value,
            _ => panic!(),
        }
    }};
}

enum MethodKind {
    Request,
    Notification,
}

struct MethodMeta {
    pub name: String,
    pub kind: MethodKind,
}

impl MethodMeta {
    pub fn parse(attr: &Attribute) -> Self {
        let meta = attr.parse_meta().unwrap();
        let nested = unwrap!(meta, Meta::List(x) => x.nested);
        let name = unwrap!(&nested[0], NestedMeta::Lit(Lit::Str(x)) => x.value());
        let kind = {
            let lit = unwrap!(&nested[1], NestedMeta::Meta(Meta::NameValue(x)) => &x.lit);
            let kind = unwrap!(lit, Lit::Str(x) => x.value());
            match kind.as_str() {
                "request" => MethodKind::Request,
                "notification" => MethodKind::Notification,
                _ => panic!(
                    "Invalid method kind. Valid options are \"request\" and \"notification\""
                ),
            }
        };

        Self { name, kind }
    }
}

#[derive(Debug, FromMeta)]
struct JsonRpcClientArgs {
    ident: Ident,

    #[darling(default)]
    keep_trait: bool,
}

#[proc_macro_attribute]
pub fn jsonrpc_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn jsonrpc_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_: ItemTrait = parse_macro_input!(item);
    let trait_ident = &trait_.ident;
    let attr: AttributeArgs = parse_macro_input!(attr);
    let args: JsonRpcClientArgs = match JsonRpcClientArgs::from_list(&attr) {
        Ok(args) => args,
        Err(why) => {
            return TokenStream::from(why.write_errors());
        }
    };

    let trait_def = if args.keep_trait {
        let doc = format!("Generated client implementation of `{}`.", trait_ident);
        quote! {
            #trait_
            #[doc = #doc]
        }
    } else {
        let trait_doc = trait_
            .attrs
            .iter()
            .filter(|attr| attr.path.get_ident().unwrap() == "doc");
        quote! {
            #(#trait_doc)*
        }
    };

    let struct_ident = args.ident;
    let stubs = generate_client_stubs(&trait_.items);
    let struct_def = quote! {
        #[derive(Debug, Clone)]
        pub struct #struct_ident {
            client: std::sync::Arc<Client>
        }

        impl #struct_ident
        {
            pub fn new(output: futures::channel::mpsc::Sender<String>) -> Self {
                Self {
                    client: std::sync::Arc::new(Client::new(output)),
                }
            }

            #(#stubs)*
        }

        #[async_trait::async_trait]
        impl ResponseHandler for #struct_ident
        {
            async fn handle(&self, response: Response) -> () {
                self.client.handle(response).await
            }
        }
    };

    let tokens = quote! {
        #trait_def
        #struct_def
    };

    tokens.into()
}

fn generate_client_stubs(items: &Vec<TraitItem>) -> Vec<TokenStream2> {
    let mut stubs = Vec::new();
    for item in items {
        let method = unwrap!(item, TraitItem::Method(x) => x);
        let sig = &method.sig;
        let ident = &sig.ident;
        let param = unwrap!(&sig.inputs[1], FnArg::Typed(x) => x);
        let param_pat = &param.pat;
        let output = &sig.output;

        let attrs = &method.attrs;
        let method_attr = attrs
            .iter()
            .find(|attr| attr.path.get_ident().unwrap() == "jsonrpc_method")
            .expect("Expected jsonrpc_method attribute");
        let meta = MethodMeta::parse(method_attr);
        let name = &meta.name;

        let stub = match meta.kind {
            MethodKind::Request => quote!(
                #(#attrs)*
                pub async fn #ident(&self, #param) #output {
                    let result = self.client.send_request(#name.to_owned(), #param_pat).await?;
                    serde_json::from_value(result).map_err(|_| Error::deserialize_error())
                }
            ),
            MethodKind::Notification => quote!(
                #(#attrs)*
                pub async fn #ident(&self, #param) {
                    self.client.send_notification(#name.to_owned(), #param_pat).await
                }
            ),
        };

        stubs.push(stub);
    }

    stubs
}
