mod client;
mod error;
mod method;
mod server;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemTrait};

#[proc_macro_attribute]
pub fn jsonrpc_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn jsonrpc_server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_: ItemTrait = parse_macro_input!(item);
    match crate::server::jsonrpc_server(trait_) {
        Ok(tokens) => tokens,
        Err(why) => why.into(),
    }
}

#[proc_macro_attribute]
pub fn jsonrpc_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_: ItemTrait = parse_macro_input!(item);
    let attr: AttributeArgs = parse_macro_input!(attr);
    match crate::client::jsonrpc_client(attr, trait_) {
        Ok(tokens) => tokens,
        Err(why) => why.into(),
    }
}
