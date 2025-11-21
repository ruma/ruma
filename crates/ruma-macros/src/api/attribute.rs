//! Details of the `#[ruma_api(...)]` attributes.

use syn::{
    Ident, Token, Type,
    parse::{Parse, ParseStream},
};

mod kw {
    syn::custom_keyword!(body);
    syn::custom_keyword!(raw_body);
    syn::custom_keyword!(path);
    syn::custom_keyword!(query);
    syn::custom_keyword!(query_all);
    syn::custom_keyword!(header);
    syn::custom_keyword!(error);
    syn::custom_keyword!(manual_body_serde);
    syn::custom_keyword!(status);
}

pub enum ResponseMeta {
    NewtypeBody,
    RawBody,
    Header(Ident),
}

impl Parse for ResponseMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::body) {
            let _: kw::body = input.parse()?;
            Ok(Self::NewtypeBody)
        } else if lookahead.peek(kw::raw_body) {
            let _: kw::raw_body = input.parse()?;
            Ok(Self::RawBody)
        } else if lookahead.peek(kw::header) {
            let _: kw::header = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::Header)
        } else {
            Err(lookahead.error())
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum DeriveResponseMeta {
    ManualBodySerde,
    Error(Type),
    Status(Ident),
}

impl Parse for DeriveResponseMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::manual_body_serde) {
            let _: kw::manual_body_serde = input.parse()?;
            Ok(Self::ManualBodySerde)
        } else if lookahead.peek(kw::error) {
            let _: kw::error = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::Error)
        } else if lookahead.peek(kw::status) {
            let _: kw::status = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::Status)
        } else {
            Err(lookahead.error())
        }
    }
}
