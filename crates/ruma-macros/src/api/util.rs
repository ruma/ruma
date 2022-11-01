//! Functions to aid the `Api::to_tokens` method.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{visit::Visit, Lifetime, Type};

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
