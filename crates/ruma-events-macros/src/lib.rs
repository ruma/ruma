#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! A procedural macro for generating [ruma-events] events.
//!
//! See the documentation for the individual macros for usage details.
//!
//! [ruma-events]: https://github.com/ruma/ruma/tree/main/ruma-events

#![warn(missing_docs)]

use event_parse::EventEnumInput;
use proc_macro::TokenStream;
use proc_macro2 as pm2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

use self::{
    event::expand_event, event_content::expand_event_content, event_enum::expand_event_enum,
    event_type::expand_event_type_enum,
};

mod event;
mod event_content;
mod event_enum;
mod event_parse;
mod event_type;

/// Generates an enum to represent the various Matrix event types.
///
/// This macro also implements the necessary traits for the type to serialize and deserialize
/// itself.
///
/// # Examples
///
/// ```ignore
/// # // HACK: This is "ignore" because of cyclical dependency drama.
/// use ruma_events_macros::event_enum;
///
/// event_enum! {
///     enum ToDevice {
///         "m.any.event",
///         "m.other.event",
///     }
///
///     enum State {
///         "m.more.events",
///         "m.different.event",
///     }
/// }
/// ```
/// (The enum name has to be a valid identifier for `<EventKind as Parse>::parse`)
//// TODO: Change above (`<EventKind as Parse>::parse`) to [] after fully qualified syntax is
//// supported:  https://github.com/rust-lang/rust/issues/74563
#[proc_macro]
pub fn event_enum(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();

    let event_enum_input = syn::parse_macro_input!(input as EventEnumInput);
    let enums = event_enum_input
        .enums
        .iter()
        .map(expand_event_enum)
        .collect::<syn::Result<pm2::TokenStream>>();
    let event_types = expand_event_type_enum(event_enum_input, ruma_events);
    event_types
        .and_then(|types| {
            enums.map(|mut enums| {
                enums.extend(types);
                enums
            })
        })
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Generates an implementation of `ruma_events::EventContent`.
#[proc_macro_derive(EventContent, attributes(ruma_event))]
pub fn derive_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_event_content(&input, &ruma_events).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Generates implementations needed to serialize and deserialize Matrix events.
#[proc_macro_derive(Event, attributes(ruma_event))]
pub fn derive_state_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

pub(crate) fn import_ruma_events() -> pm2::TokenStream {
    if let Ok(FoundCrate::Name(possibly_renamed)) = crate_name("ruma-events") {
        let import = Ident::new(&possibly_renamed, pm2::Span::call_site());
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(possibly_renamed)) = crate_name("ruma") {
        let import = Ident::new(&possibly_renamed, pm2::Span::call_site());
        quote! { ::#import::events }
    } else {
        quote! { ::ruma_events }
    }
}
