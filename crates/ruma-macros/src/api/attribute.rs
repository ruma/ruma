//! Details of the `#[ruma_api(...)]` attributes.

use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Token, Type,
};

mod kw {
    syn::custom_keyword!(body);
    syn::custom_keyword!(raw_body);
    syn::custom_keyword!(path);
    syn::custom_keyword!(query);
    syn::custom_keyword!(query_map);
    syn::custom_keyword!(header);
    syn::custom_keyword!(authentication);
    syn::custom_keyword!(method);
    syn::custom_keyword!(error_ty);
    syn::custom_keyword!(unstable);
    syn::custom_keyword!(r0);
    syn::custom_keyword!(stable);
    syn::custom_keyword!(manual_body_serde);
}

pub enum RequestMeta {
    NewtypeBody,
    RawBody,
    Path,
    Query,
    QueryMap,
    Header(Ident),
}

impl Parse for RequestMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::body) {
            let _: kw::body = input.parse()?;
            Ok(Self::NewtypeBody)
        } else if lookahead.peek(kw::raw_body) {
            let _: kw::raw_body = input.parse()?;
            Ok(Self::RawBody)
        } else if lookahead.peek(kw::path) {
            let _: kw::path = input.parse()?;
            Ok(Self::Path)
        } else if lookahead.peek(kw::query) {
            let _: kw::query = input.parse()?;
            Ok(Self::Query)
        } else if lookahead.peek(kw::query_map) {
            let _: kw::query_map = input.parse()?;
            Ok(Self::QueryMap)
        } else if lookahead.peek(kw::header) {
            let _: kw::header = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::Header)
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum DeriveRequestMeta {
    Authentication(Type),
    Method(Type),
    ErrorTy(Type),
    UnstablePath(LitStr),
    R0Path(LitStr),
    StablePath(LitStr),
}

impl Parse for DeriveRequestMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::authentication) {
            let _: kw::authentication = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::Authentication)
        } else if lookahead.peek(kw::method) {
            let _: kw::method = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::Method)
        } else if lookahead.peek(kw::error_ty) {
            let _: kw::error_ty = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::ErrorTy)
        } else if lookahead.peek(kw::unstable) {
            let _: kw::unstable = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::UnstablePath)
        } else if lookahead.peek(kw::r0) {
            let _: kw::r0 = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::R0Path)
        } else if lookahead.peek(kw::stable) {
            let _: kw::stable = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::StablePath)
        } else {
            Err(lookahead.error())
        }
    }
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
    ErrorTy(Type),
}

impl Parse for DeriveResponseMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::manual_body_serde) {
            let _: kw::manual_body_serde = input.parse()?;
            Ok(Self::ManualBodySerde)
        } else if lookahead.peek(kw::error_ty) {
            let _: kw::error_ty = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(Self::ErrorTy)
        } else {
            Err(lookahead.error())
        }
    }
}
