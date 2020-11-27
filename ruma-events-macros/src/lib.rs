//! Crate `ruma_events_macros` provides a procedural macro for generating
//! [ruma-events] events.
//!
//! See the documentation for the individual macros for usage details.
//!
//! [ruma-events]: https://github.com/ruma/ruma/tree/main/ruma-events

#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs)]

use proc_macro::TokenStream;
use proc_macro2 as pm2;
use proc_macro_crate::crate_name;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

use self::{
    event::expand_event,
    event_content::{
        expand_basic_event_content, expand_ephemeral_room_event_content, expand_event_content,
        expand_message_event_content, expand_room_event_content, expand_state_event_content,
    },
    event_enum::expand_event_enum,
    event_parse::EventEnumInput,
};

mod event;
mod event_content;
mod event_enum;
mod event_parse;
/// Generates an enum to represent the various Matrix event types.
///
/// This macro also implements the necessary traits for the type to serialize and deserialize
/// itself.
///
/// # Examples
///
/// ```ignore
/// use ruma_events_macros::event_enum;
///
/// event_enum! {
///     name: AnyBarEvent, // `BarEvent` has to be a valid type at `::ruma_events::BarEvent`
///     events: [
///         "m.any.event",
///         "m.other.event",
///     ]
/// }
/// ```
#[proc_macro]
pub fn event_enum(input: TokenStream) -> TokenStream {
    let event_enum_input = syn::parse_macro_input!(input as EventEnumInput);
    expand_event_enum(event_enum_input).unwrap_or_else(|err| err.to_compile_error()).into()
}

/// Generates an implementation of `ruma_events::EventContent`.
#[proc_macro_derive(EventContent, attributes(ruma_event))]
pub fn derive_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_event_content(&input, true, &ruma_events)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::BasicEventContent` and it's super traits.
#[proc_macro_derive(BasicEventContent, attributes(ruma_event))]
pub fn derive_basic_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_basic_event_content(&input, &ruma_events)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::RoomEventContent` and it's super traits.
#[proc_macro_derive(RoomEventContent, attributes(ruma_event))]
pub fn derive_room_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_room_event_content(&input, &ruma_events)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::MessageEventContent` and it's super traits.
#[proc_macro_derive(MessageEventContent, attributes(ruma_event))]
pub fn derive_message_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_message_event_content(&input, &ruma_events)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::StateEventContent` and it's super traits.
#[proc_macro_derive(StateEventContent, attributes(ruma_event))]
pub fn derive_state_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_state_event_content(&input, &ruma_events)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::EphemeralRoomEventContent` and it's super traits.
#[proc_macro_derive(EphemeralRoomEventContent, attributes(ruma_event))]
pub fn derive_ephemeral_room_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_ephemeral_room_event_content(&input, &ruma_events)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates implementations needed to serialize and deserialize Matrix events.
#[proc_macro_derive(Event, attributes(ruma_event))]
pub fn derive_state_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event(input).unwrap_or_else(|err| err.to_compile_error()).into()
}

pub(crate) fn import_ruma_events() -> pm2::TokenStream {
    if let Ok(possibly_renamed) = crate_name("ruma-events") {
        let import = Ident::new(&possibly_renamed, pm2::Span::call_site());
        quote! { ::#import }
    } else if let Ok(possibly_renamed) = crate_name("ruma") {
        let import = Ident::new(&possibly_renamed, pm2::Span::call_site());
        quote! { ::#import::events }
    } else {
        quote! { ::ruma_events }
    }
}
