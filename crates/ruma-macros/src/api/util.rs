//! Functions to aid the `Api::to_tokens` method.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn map_option_literal<T: ToTokens>(ver: &Option<T>) -> TokenStream {
    match ver {
        Some(v) => quote! { ::std::option::Option::Some(#v) },
        None => quote! { ::std::option::Option::None },
    }
}

pub fn is_valid_endpoint_path(string: &str) -> bool {
    string.as_bytes().iter().all(|b| (0x21..=0x7E).contains(b))
}
