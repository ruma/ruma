//! Crate ruma-api-macros provides a procedural macro for easily generating
//! [ruma-api](https://github.com/ruma/ruma-api)-compatible endpoints.
//!
//! This crate should never be used directly; instead, use it through the
//! re-exports in ruma-api. Also note that for technical reasons, the
//! `ruma_api!` macro is only documented in ruma-api, not here.

#![deny(missing_copy_implementations, missing_debug_implementations)]
#![allow(clippy::cognitive_complexity)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]
#![recursion_limit = "256"]

extern crate proc_macro;

use std::convert::TryFrom as _;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

use self::{
    api::{Api, RawApi},
    send_recv::expand_send_recv,
};

mod api;
mod send_recv;

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let raw_api = parse_macro_input!(input as RawApi);
    match Api::try_from(raw_api) {
        Ok(api) => api.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(SendRecv, attributes(wrap_incoming, incoming_no_deserialize))]
pub fn derive_send_recv(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_send_recv(input).unwrap_or_else(|err| err.to_compile_error()).into()
}
