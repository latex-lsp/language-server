use crate::error::{Error, Result};
use darling::FromMeta;
use syn::{spanned::Spanned, *};

#[derive(Debug, FromMeta)]
pub enum MethodKind {
    Request,
    Notification,
}

#[derive(Debug, FromMeta)]
pub struct JsonRpcMethodArgs {
    pub name: String,
    pub kind: MethodKind,
}

impl JsonRpcMethodArgs {
    pub fn parse(method: &TraitItemMethod) -> Result<Option<JsonRpcMethodArgs>> {
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
