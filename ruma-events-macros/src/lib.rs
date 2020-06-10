//! Crate `ruma_events_macros` provides a procedural macro for generating
//! [ruma-events](https://github.com/ruma/ruma-events) events.
//!
//! See the documentation for the invidiual macros for usage details.
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use self::{
    content_enum::{expand_content_enum, parse::ContentEnumInput},
    event::expand_event,
    event_content::{
        expand_basic_event_content, expand_ephemeral_room_event_content, expand_event_content,
        expand_message_event_content, expand_room_event_content, expand_state_event_content,
    },
    event_enum::{expand_event_enum, EventEnumInput},
};

mod content_enum;
mod event;
mod event_content;
mod event_enum;

/// Generates an enum to represent the various Matrix event types.
///
/// This macro also implements the necessary traits for the type to serialize and deserialize
/// itself.
// TODO more docs/example
#[proc_macro]
pub fn event_enum(input: TokenStream) -> TokenStream {
    let event_enum_input = syn::parse_macro_input!(input as EventEnumInput);
    expand_event_enum(event_enum_input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates a content enum to represent the various Matrix event types.
///
/// This macro also implements the necessary traits for the type to serialize and deserialize
/// itself.
// TODO more docs/example
#[proc_macro]
pub fn event_content_enum(input: TokenStream) -> TokenStream {
    let content_enum_input = syn::parse_macro_input!(input as ContentEnumInput);
    expand_content_enum(content_enum_input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::EventContent`.
#[proc_macro_derive(EventContent, attributes(ruma_event))]
pub fn derive_event_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event_content(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::BasicEventContent` and it's super traits.
#[proc_macro_derive(BasicEventContent, attributes(ruma_event))]
pub fn derive_basic_event_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_basic_event_content(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::RoomEventContent` and it's super traits.
#[proc_macro_derive(RoomEventContent, attributes(ruma_event))]
pub fn derive_room_event_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_room_event_content(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::MessageEventContent` and it's super traits.
#[proc_macro_derive(MessageEventContent, attributes(ruma_event))]
pub fn derive_message_event_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_message_event_content(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::StateEventContent` and it's super traits.
#[proc_macro_derive(StateEventContent, attributes(ruma_event))]
pub fn derive_state_event_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_state_event_content(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates an implementation of `ruma_events::EphemeralRoomEventContent` and it's super traits.
#[proc_macro_derive(EphemeralRoomEventContent, attributes(ruma_event))]
pub fn derive_ephemeral_room_event_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_ephemeral_room_event_content(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generates implementations needed to serialize and deserialize Matrix events.
#[proc_macro_derive(Event, attributes(ruma_event))]
pub fn derive_state_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
