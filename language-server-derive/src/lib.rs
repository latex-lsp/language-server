use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{export::TokenStream2, spanned::Spanned, *};

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

impl JsonRpcMethodArgs {
    fn parse(method: &TraitItemMethod) -> Result<Option<JsonRpcMethodArgs>> {
        let attrs = &method.attrs;
        let method_attr = attrs
            .iter()
            .find(|attr| attr.path.is_ident("jsonrpc_method"));
        if method_attr.is_none() {
            return Ok(None);
        }

        if method.sig.inputs.is_empty() || method.sig.inputs.len() < 2 {
            let span = method.sig.inputs.span();
            let error = syn::Error::new(span, "expected &self and params argument");
            return Err(Error::Syn(error));
        }

        if let FnArg::Typed(param) = &method.sig.inputs[0] {
            let error = syn::Error::new(param.span(), "expected &self argument");
            return Err(Error::Syn(error));
        }

        let args = JsonRpcMethodArgs::from_meta(&method_attr.unwrap().parse_meta()?)?;
        Ok(Some(args))
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
pub fn jsonrpc_server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_: ItemTrait = parse_macro_input!(item);
    let (requests, notifications) = match generate_server_skeletons(&trait_.items) {
        Ok((reqs, nots)) => (reqs, nots),
        Err(Error::Syn(why)) => return why.to_compile_error().into(),
        Err(Error::Darling(why)) => return why.write_errors().into(),
    };

    let tokens = quote! {
        #trait_

        #[async_trait::async_trait]
        impl<S: LanguageServer + Sync> RequestHandler for S {
            async fn handle_request(&self, request: Request, client: LanguageClient) -> Response {
                match request.method.as_str() {
                    #requests,
                    _ => {
                        Response::error(Error::method_not_found_error(), Some(request.id))
                    }
                }
            }

            async fn handle_notification(&self, notification: Notification, client: LanguageClient) {
                match notification.method.as_str() {
                    #notifications,
                    _ => log::warn!("{}: {}", "Method not found", notification.method),
                }
            }
        }
    };

    tokens.into()
}

fn generate_server_skeletons(items: &Vec<TraitItem>) -> Result<(TokenStream2, TokenStream2)> {
    let mut requests = Vec::new();
    let mut notifications = Vec::new();

    for item in items {
        let method = match item {
            TraitItem::Method(method) => method,
            _ => continue,
        };

        let args = match JsonRpcMethodArgs::parse(method)? {
            Some(args) => args,
            None => continue,
        };

        let ident = &method.sig.ident;
        let name = args.name;

        match args.kind {
            MethodKind::Request => requests.push(quote!(
                #name => {
                    let handle = |json| async move {
                        let params = serde_json::from_value(json).map_err(|_| Error::deserialize_error())?;
                        let result = self.#ident(params, client).await?;
                        Ok(result)
                    };

                    match handle(request.params).await {
                        Ok(result) => Response::result(json!(result), request.id),
                        Err(error) => Response::error(error, Some(request.id)),
                    }
                }
            )),
            MethodKind::Notification => notifications.push(quote!(
                #name => {
                    let error = Error::deserialize_error().message;
                    let params = serde_json::from_value(notification.params).expect(&error);
                    self.#ident(params, client).await;
                }
            )),
        };
    }

    Ok((quote! { #(#requests)* }, quote! { #(#notifications)* }))
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
    let args = match JsonRpcMethodArgs::parse(method)? {
        Some(args) => args,
        None => return Ok(None),
    };

    let attrs = &method.attrs;
    let ident = &method.sig.ident;
    let param = match &method.sig.inputs[1] {
        FnArg::Typed(param) => param,
        FnArg::Receiver(_) => unreachable!(),
    };
    let param_pat = &param.pat;
    let output = &method.sig.output;
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
