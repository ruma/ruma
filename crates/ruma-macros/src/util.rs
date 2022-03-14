use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{Ident, LitStr};

pub(crate) fn import_ruma_common() -> TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-common") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk-appservice") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma }
    } else {
        quote! { ::ruma_common }
    }
}

/// CamelCase's a field ident like "foo_bar" to "FooBar".
pub(crate) fn to_camel_case(name: &Ident) -> Ident {
    let span = name.span();
    let name = name.to_string();

    let s: String = name
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();
    Ident::new(&s, span)
}

/// Splits the given string on `.` and `_` removing the `m.` then camel casing to give a Rust type
/// name.
pub(crate) fn m_prefix_name_to_type_name(name: &LitStr) -> syn::Result<Ident> {
    let span = name.span();
    let name = name.value();

    let name = name.strip_prefix("m.").ok_or_else(|| {
        syn::Error::new(
            span,
            format!("well-known matrix events have to start with `m.` found `{}`", name),
        )
    })?;

    let s: String = name
        .strip_suffix(".*")
        .unwrap_or(name)
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    Ok(Ident::new(&s, span))
}
