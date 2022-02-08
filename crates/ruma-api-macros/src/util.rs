//! Functions to aid the `Api::to_tokens` method.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, visit::Visit, AttrStyle, Attribute, Lifetime, NestedMeta, Type};

pub fn import_ruma_api() -> TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-api") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{}", name);
        quote! { ::#import::api }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma::api }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk-appservice") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma::api }
    } else {
        quote! { ::ruma_api }
    }
}

pub fn map_option_literal<T: ToTokens>(ver: &Option<T>) -> TokenStream {
    match ver {
        Some(v) => quote! { ::std::option::Option::Some(#v) },
        None => quote! { ::std::option::Option::None },
    }
}

pub fn is_valid_endpoint_path(string: &str) -> bool {
    string.as_bytes().iter().all(|b| (0x21..=0x7E).contains(b))
}

pub fn collect_lifetime_idents(lifetimes: &mut BTreeSet<Lifetime>, ty: &Type) {
    struct Visitor<'lt>(&'lt mut BTreeSet<Lifetime>);
    impl<'ast> Visit<'ast> for Visitor<'_> {
        fn visit_lifetime(&mut self, lt: &'ast Lifetime) {
            self.0.insert(lt.clone());
        }
    }

    Visitor(lifetimes).visit_type(ty)
}

pub fn is_cfg_attribute(attr: &Attribute) -> bool {
    matches!(attr.style, AttrStyle::Outer) && attr.path.is_ident("cfg")
}

pub fn all_cfgs_expr(cfgs: &[Attribute]) -> Option<TokenStream> {
    let sub_cfgs: Vec<_> = cfgs.iter().filter_map(extract_cfg).collect();
    (!sub_cfgs.is_empty()).then(|| quote! { all( #(#sub_cfgs),* ) })
}

pub fn all_cfgs(cfgs: &[Attribute]) -> Option<Attribute> {
    let cfg_expr = all_cfgs_expr(cfgs)?;
    Some(parse_quote! { #[cfg( #cfg_expr )] })
}

pub fn extract_cfg(attr: &Attribute) -> Option<NestedMeta> {
    if !attr.path.is_ident("cfg") {
        return None;
    }

    let meta = attr.parse_meta().expect("cfg attribute can be parsed to syn::Meta");
    let mut list = match meta {
        syn::Meta::List(l) => l,
        _ => panic!("unexpected cfg syntax"),
    };

    assert!(list.path.is_ident("cfg"), "expected cfg attributes only");
    assert_eq!(list.nested.len(), 1, "expected one item inside cfg()");

    Some(list.nested.pop().unwrap().into_value())
}
