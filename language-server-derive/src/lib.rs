use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{export::TokenStream2, *};

enum Error {
    Syn(syn::Error),
    Darling(darling::Error),
}

impl From<syn::Error> for Error {
    fn from(error: syn::Error) -> Self {
        Error::Syn(error)
    }
}

impl From<darling::Error> for Error {
    fn from(error: darling::Error) -> Self {
        Error::Darling(error)
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, FromMeta)]
enum MethodKind {
    Request,
    Notification,
}

#[derive(Debug, FromMeta)]
struct JsonRpcMethodArgs {
    pub name: String,
    pub kind: MethodKind,
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
    let attr: AttributeArgs = parse_macro_input!(attr);
    let args = match JsonRpcClientArgs::from_list(&attr) {
        Ok(args) => args,
        Err(why) => {
            return TokenStream::from(why.write_errors());
        }
    };

    let trait_def = if args.keep_trait {
        let doc = format!("Generated client implementation of `{}`.", &trait_.ident);
        quote! {
            #trait_
            #[doc = #doc]
        }
    } else {
        let trait_doc = trait_.attrs.iter().filter(|attr| attr.path.is_ident("doc"));
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

            #stubs
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

fn generate_client_stubs(items: &Vec<TraitItem>) -> TokenStream2 {
    let mut stubs = Vec::new();
    for item in items {
        let method = match item {
            TraitItem::Method(method) => method,
            _ => continue,
        };

        let stub = match create_client_stub(method) {
            Ok(Some(stub)) => stub,
            Ok(None) => quote! {},
            Err(Error::Syn(why)) => why.to_compile_error(),
            Err(Error::Darling(why)) => why.write_errors(),
        };

        stubs.push(stub);
    }

    quote! {
        #(#stubs)*
    }
}

fn create_client_stub(method: &TraitItemMethod) -> Result<Option<TokenStream2>> {
    let attrs = &method.attrs;
    let method_attr = attrs
        .iter()
        .find(|attr| attr.path.is_ident("jsonrpc_method"));

    if method_attr.is_none() {
        return Ok(None);
    }

    if method.sig.inputs.is_empty() || method.sig.inputs.len() < 2 {
        let span = method.sig.paren_token.span;
        let error = syn::Error::new(span, "expected &self and params argument");
        return Err(Error::Syn(error));
    }

    let ident = &method.sig.ident;
    let param = match &method.sig.inputs[1] {
        FnArg::Typed(param) => param,
        FnArg::Receiver(_) => unreachable!(),
    };

    let param_pat = &param.pat;
    let output = &method.sig.output;
    let args = JsonRpcMethodArgs::from_meta(&method_attr.unwrap().parse_meta()?)?;
    let name = args.name;

    let stub = match args.kind {
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

    Ok(Some(stub))
}
