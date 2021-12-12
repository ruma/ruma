//! Functions to aid the `Api::to_tokens` method.

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_quote, visit::Visit, AttrStyle, Attribute, Field, Lifetime, NestedMeta, Type};

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

fn all_cfgs(cfgs: &[Attribute]) -> Option<Attribute> {
    let cfg_expr = all_cfgs_expr(cfgs)?;
    Some(parse_quote! { #[cfg( #cfg_expr )] })
}

fn extract_cfg(attr: &Attribute) -> Option<NestedMeta> {
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

/// The combination of every fields unique lifetime annotation.
pub fn all_lifetimes<'a>(
    fields: impl IntoIterator<Item = &'a Field>,
) -> BTreeMap<Lifetime, Option<Attribute>> {
    struct Visitor<'lt> {
        field_cfg: Option<Attribute>,
        lifetimes: &'lt mut BTreeMap<Lifetime, Option<Attribute>>,
    }

    impl<'ast> Visit<'ast> for Visitor<'_> {
        fn visit_lifetime(&mut self, lt: &'ast Lifetime) {
            match self.lifetimes.entry(lt.clone()) {
                Entry::Vacant(v) => {
                    v.insert(self.field_cfg.clone());
                }
                Entry::Occupied(mut o) => {
                    let lifetime_cfg = o.get_mut();

                    // If at least one field uses this lifetime and has no cfg attribute, we
                    // don't need a cfg attribute for the lifetime either.
                    *lifetime_cfg = Option::zip(lifetime_cfg.as_ref(), self.field_cfg.as_ref())
                        .map(|(a, b)| {
                            let expr_a = extract_cfg(a);
                            let expr_b = extract_cfg(b);
                            parse_quote! { #[cfg( any( #expr_a, #expr_b ) )] }
                        });
                }
            }
        }
    }

    let mut lifetimes = BTreeMap::new();
    for field in fields {
        let field_cfg = if field.attrs.is_empty() { None } else { all_cfgs(&field.attrs) };
        Visitor { lifetimes: &mut lifetimes, field_cfg }.visit_type(&field.ty);
    }

    lifetimes
}
