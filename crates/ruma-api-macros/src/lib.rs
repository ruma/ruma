#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! A procedural macro for easily generating [ruma-api]-compatible endpoints.
//!
//! This crate should never be used directly; instead, use it through the
//! re-exports in ruma-api. Also note that for technical reasons, the
//! `ruma_api!` macro is only documented in ruma-api, not here.
//!
//! [ruma-api]: https://github.com/ruma/ruma/tree/main/ruma-api

#![recursion_limit = "256"]

use proc_macro::TokenStream;
use syn::parse_macro_input;

use self::api::Api;

mod api;
mod util;

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let api = parse_macro_input!(input as Api);
    api::expand_all(api).unwrap_or_else(syn::Error::into_compile_error).into()
}
