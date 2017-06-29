//! Crate `ruma-api-macros` provides a procedural macro for easily generating `ruma-api` endpoints.

#![deny(missing_debug_implementations)]
#![feature(proc_macro)]
#![recursion_limit="128"]

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate ruma_api;
extern crate syn;
#[macro_use] extern crate synom;

use proc_macro::TokenStream;

use quote::{ToTokens, Tokens};

use api::Api;
use parse::parse_entries;

mod api;
mod parse;

/// Generates a `ruma-api` endpoint.
#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let entries = parse_entries(&input.to_string()).expect("ruma_api! failed to parse input");

    let api = Api::from(entries);

    let mut tokens = Tokens::new();

    api.to_tokens(&mut tokens);

    tokens.parse().expect("ruma_api! failed to parse output tokens as a TokenStream")
}
