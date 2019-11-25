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
    derive_outgoing::expand_derive_outgoing,
};

mod api;
mod derive_outgoing;

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let raw_api = parse_macro_input!(input as RawApi);
    match Api::try_from(raw_api) {
        Ok(api) => api.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive the `Outgoing` trait, possibly generating an 'Incoming' version of the struct this
/// derive macro is used on. Specifically, if no `#[wrap_incoming]` attribute is used on any of the
/// fields of the struct, this simple implementation will be generated:
///
/// ```ignore
/// impl Outgoing for MyType {
///     type Incoming = Self;
/// }
/// ```
///
/// If, however, `#[wrap_incoming]` is used (which is the only reason you should ever use this
/// derive macro manually), a new struct `IncomingT` (where `T` is the type this derive is used on)
/// is generated, with all of the fields with `#[wrap_incoming]` replaced:
///
/// ```ignore
/// #[derive(Outgoing)]
/// struct MyType {
///     pub foo: Foo,
///     #[wrap_incoming]
///     pub bar: Bar,
///     #[wrap_incoming(Baz)]
///     pub baz: Option<Baz>,
///     #[wrap_incoming(with EventResult)]
///     pub x: XEvent,
///     #[wrap_incoming(YEvent with EventResult)]
///     pub ys: Vec<YEvent>,
/// }
///
/// // generated
/// struct IncomingMyType {
///     pub foo: Foo,
///     pub bar: IncomingBar,
///     pub baz: Option<IncomingBaz>,
///     pub x: EventResult<XEvent>,
///     pub ys: Vec<EventResult<YEvent>>,
/// }
/// ```
#[proc_macro_derive(Outgoing, attributes(wrap_incoming, incoming_no_deserialize))]
pub fn derive_send_recv(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_outgoing(input).unwrap_or_else(|err| err.to_compile_error()).into()
}
