//! Crate ruma-api-macros provides a procedural macro for easily generating
//! [ruma-api](https://github.com/ruma/ruma-api)-compatible endpoints.
//!
//! This crate should never be used directly; instead, use it through the
//! re-exports in ruma-api. Also note that for technical reasons, the
//! `ruma_api!` macro is only documented in ruma-api, not here.

#![allow(clippy::cognitive_complexity, clippy::unnested_or_patterns)]
#![recursion_limit = "256"]

extern crate proc_macro;

use std::convert::TryFrom as _;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use self::api::{Api, RawApi};

mod api;

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let raw_api = parse_macro_input!(input as RawApi);
    match Api::try_from(raw_api) {
        Ok(api) => api.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}
