use crate::{
    error::Result,
    method::{JsonRpcMethodArgs, MethodKind},
};
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{export::TokenStream2, *};

#[derive(Debug, FromMeta)]
struct JsonRpcClientArgs {
    ident: Ident,
}

pub fn jsonrpc_client(attr: AttributeArgs, trait_: ItemTrait) -> Result<TokenStream> {
    let args = JsonRpcClientArgs::from_list(&attr)?;
    let trait_ident = &trait_.ident;
    let struct_ident = args.ident;
    let stubs = generate_client_stubs(&trait_.items)?;
    let tokens = quote! {
        #trait_

        #[derive(Debug)]
        pub struct #struct_ident {
            client: Client
        }

        impl #struct_ident
        {
            pub fn new(output: futures::channel::mpsc::Sender<Message>) -> Self {
                Self {
                    client: Client::new(output),
                }
            }
        }

        #[async_trait::async_trait]
        impl #trait_ident for #struct_ident
        {
            #stubs
        }

        #[async_trait::async_trait]
        impl ResponseHandler for #struct_ident
        {
            async fn handle(&self, response: Response) {
                self.client.handle(response).await;
            }
        }
    };

    Ok(tokens.into())
}

fn generate_client_stubs(items: &Vec<TraitItem>) -> Result<TokenStream2> {
    let mut stubs = Vec::new();
    for item in items {
        let method = match item {
            TraitItem::Method(method) => method,
            _ => continue,
        };
        let args = match JsonRpcMethodArgs::parse(method)? {
            Some(args) => args,
            None => continue,
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
                async fn #ident(&self, #param) #output {
                    let result = self.client.send_request(#name.to_owned(), #param_pat).await?;
                    serde_json::from_value(result).map_err(|_| Error::deserialize_error())
                }
            ),
            MethodKind::Notification => quote!(
                #(#attrs)*
                async fn #ident(&self, #param) {
                    self.client.send_notification(#name.to_owned(), #param_pat).await
                }
            ),
        };

        stubs.push(stub);
    }

    Ok(quote! { #(#stubs)* })
}
