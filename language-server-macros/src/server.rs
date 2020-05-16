use crate::{
    error::Result,
    method::{JsonRpcMethodArgs, MethodKind},
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{export::TokenStream2, *};

pub fn jsonrpc_server(trait_: ItemTrait) -> Result<TokenStream> {
    let (requests, notifications) = generate_server_skeletons(&trait_.items)?;
    let tokens = quote! {
        #trait_

        #[async_trait::async_trait]
        impl<S, C> RequestHandler<C> for S
        where
            S: LanguageServer + Sync,
            C: LanguageClient,
        {
            async fn handle_request(&self, request: Request, client: Arc<C>) -> Response {
                match request.method.as_str() {
                    #requests,
                    _ => {
                        Response::error(Error::method_not_found_error(), Some(request.id))
                    }
                }
            }

            async fn handle_notification(&self, notification: Notification, client: Arc<C>) {
                match notification.method.as_str() {
                    #notifications,
                    _ => log::warn!("{}: {}", "Method not found", notification.method),
                }
            }
        }
    };

    Ok(tokens.into())
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
        let cfg_attrs = method.attrs.iter().filter(|attr| attr.path.is_ident("cfg"));

        match args.kind {
            MethodKind::Request => requests.push(quote!(
                #(#cfg_attrs)*
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
                #(#cfg_attrs)*
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
