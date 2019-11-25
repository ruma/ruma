//! Crate `ruma-api-macros` provides a procedural macro for easily generating
//! [ruma-api](https://github.com/ruma/ruma-api)-compatible endpoints.
//!
//! See the documentation for the `ruma_api!` macro for usage details.

#![deny(missing_copy_implementations, missing_debug_implementations)]
#![allow(clippy::cognitive_complexity)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]
#![recursion_limit = "256"]

extern crate proc_macro;

use std::convert::TryFrom as _;

use proc_macro::TokenStream;
use quote::ToTokens;

use crate::api::{Api, RawApi};

mod api;

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let raw_api = syn::parse_macro_input!(input as RawApi);
    match Api::try_from(raw_api) {
        Ok(api) => api.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}
