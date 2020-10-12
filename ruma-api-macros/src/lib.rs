//! Crate ruma-api-macros provides a procedural macro for easily generating
//! [ruma-api]-compatible endpoints.
//!
//! This crate should never be used directly; instead, use it through the
//! re-exports in ruma-api. Also note that for technical reasons, the
//! `ruma_api!` macro is only documented in ruma-api, not here.
//!
//! [ruma-api]: https://github.com/ruma/ruma/tree/main/ruma-api

#![allow(clippy::cognitive_complexity)]
// Remove this once https://github.com/rust-lang/rust/issues/54883 becomes stable
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::unknown_clippy_lints)]
#![recursion_limit = "256"]

use std::convert::TryFrom as _;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use self::api::{Api, RawApi};

mod api;
mod util;

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let raw_api = parse_macro_input!(input as RawApi);
    match Api::try_from(raw_api) {
        Ok(api) => api.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}
