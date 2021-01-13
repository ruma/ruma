//! Functions to aid the `Api::to_tokens` method.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{AttrStyle, Attribute, Lifetime};

/// Generates a `TokenStream` of lifetime identifiers `<'lifetime>`.
pub(crate) fn unique_lifetimes_to_tokens<'a, I: IntoIterator<Item = &'a Lifetime>>(
    lifetimes: I,
) -> TokenStream {
    let lifetimes = lifetimes.into_iter().collect::<BTreeSet<_>>();
    (!lifetimes.is_empty())
        .then(|| {
            let lifetimes = quote! { #( #lifetimes ),* };
            quote! { < #lifetimes > }
        })
        .unwrap_or_default()
}

pub(crate) fn is_valid_endpoint_path(string: &str) -> bool {
    string.as_bytes().iter().all(|b| (0x21..=0x7E).contains(b))
}

pub(crate) fn import_ruma_api() -> TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-api") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{}", name);
        quote! { ::#import::api }
    } else {
        quote! { ::ruma_api }
    }
}

pub(crate) fn is_cfg_attribute(attr: &Attribute) -> bool {
    matches!(attr.style, AttrStyle::Outer) && attr.path.is_ident("cfg")
}
