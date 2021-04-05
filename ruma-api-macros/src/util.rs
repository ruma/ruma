//! Functions to aid the `Api::to_tokens` method.

use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{AttrStyle, Attribute, Ident, Lifetime};

/// Generates a `TokenStream` of lifetime identifiers `<'lifetime>`.
pub(crate) fn unique_lifetimes_to_tokens<'a, I: Iterator<Item = &'a Lifetime>>(
    lifetimes: I,
) -> TokenStream {
    let lifetimes = lifetimes.collect::<BTreeSet<_>>();
    if lifetimes.is_empty() {
        TokenStream::new()
    } else {
        let lifetimes = quote! { #( #lifetimes ),* };
        quote! { < #lifetimes > }
    }
}

pub(crate) fn is_valid_endpoint_path(string: &str) -> bool {
    string.as_bytes().iter().all(|b| (0x21..=0x7E).contains(b))
}

pub(crate) fn import_ruma_api() -> TokenStream {
    if let Ok(FoundCrate::Name(possibly_renamed)) = crate_name("ruma-api") {
        let import = Ident::new(&possibly_renamed, Span::call_site());
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(possibly_renamed)) = crate_name("ruma") {
        let import = Ident::new(&possibly_renamed, Span::call_site());
        quote! { ::#import::api }
    } else {
        quote! { ::ruma_api }
    }
}

pub(crate) fn is_cfg_attribute(attr: &Attribute) -> bool {
    attr.style == AttrStyle::Outer && attr.path.is_ident("cfg")
}
