use proc_macro::TokenStream;

pub enum Error {
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

impl Into<TokenStream> for Error {
    fn into(self) -> TokenStream {
        match self {
            Error::Syn(why) => why.to_compile_error().into(),
            Error::Darling(why) => why.write_errors().into(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
