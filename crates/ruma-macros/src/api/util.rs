//! Functions to aid the `Api::to_tokens` method.

use std::collections::BTreeSet;

use proc_macro2::{Ident, Span, TokenStream};
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

    Visitor(lifetimes).visit_type(ty)
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

pub fn path_format_args_call(
    mut format_string: String,
    percent_encoding: &TokenStream,
) -> TokenStream {
    let mut format_args = Vec::new();

    while let Some(start_of_segment) = format_string.find(':') {
        // ':' should only ever appear at the start of a segment
        assert_eq!(&format_string[start_of_segment - 1..start_of_segment], "/");

        let end_of_segment = match format_string[start_of_segment..].find('/') {
            Some(rel_pos) => start_of_segment + rel_pos,
            None => format_string.len(),
        };

        let path_var =
            Ident::new(&format_string[start_of_segment + 1..end_of_segment], Span::call_site());
        format_args.push(quote! {
            #percent_encoding::utf8_percent_encode(
                &::std::string::ToString::to_string(&self.#path_var),
                #percent_encoding::NON_ALPHANUMERIC,
            )
        });
        format_string.replace_range(start_of_segment..end_of_segment, "{}");
    }

    quote! {
        format_args!(#format_string, #(#format_args),*)
    }
}
