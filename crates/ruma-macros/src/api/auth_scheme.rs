use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

mod kw {
    syn::custom_keyword!(None);
    syn::custom_keyword!(AccessToken);
    syn::custom_keyword!(ServerSignatures);
}

pub enum AuthScheme {
    None(kw::None),
    AccessToken(kw::AccessToken),
    ServerSignatures(kw::ServerSignatures),
}

impl Parse for AuthScheme {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::None) {
            input.parse().map(Self::None)
        } else if lookahead.peek(kw::AccessToken) {
            input.parse().map(Self::AccessToken)
        } else if lookahead.peek(kw::ServerSignatures) {
            input.parse().map(Self::ServerSignatures)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for AuthScheme {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AuthScheme::None(kw) => kw.to_tokens(tokens),
            AuthScheme::AccessToken(kw) => kw.to_tokens(tokens),
            AuthScheme::ServerSignatures(kw) => kw.to_tokens(tokens),
        }
    }
}
