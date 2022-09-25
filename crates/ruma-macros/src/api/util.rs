//! Functions to aid the `Api::to_tokens` method.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, visit::Visit, Attribute, Lifetime, NestedMeta, Type};

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

    Visitor(lifetimes).visit_type(ty);
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

pub fn path_into_format_parts(path: &str) -> Vec<String> {
    let mut parts = Vec::new();

    // Note: this assumes that `:` does not appear anywhere else in a path string
    let mut iter = path.split(':');

    // First part is always either full string, or first part until an argument
    parts.push(iter.next().expect("split gives always at least 1 element").to_owned());

    'chunks: for chunk in iter {
        for (i, c) in chunk.chars().enumerate() {
            if c == '/' {
                parts.push(chunk[i..].to_owned());

                continue 'chunks;
            }
        }

        if !chunk.contains('/') {
            parts.push("".to_owned());
        }
    }

    parts
}

#[test]
fn check_format_parts() {
    assert_eq!(path_into_format_parts("/testing"), vec!["/testing"]);
    assert_eq!(path_into_format_parts("/testing/:abc"), vec!["/testing/", ""]);
    assert_eq!(path_into_format_parts("/testing/:abc/"), vec!["/testing/", "/"]);
    assert_eq!(path_into_format_parts("/testing/:abc/:dce"), vec!["/testing/", "/", ""]);
    assert_eq!(
        path_into_format_parts("/testing/:abc/element/:dce"),
        vec!["/testing/", "/element/", ""]
    );
}
