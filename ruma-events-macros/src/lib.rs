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
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

use self::{
    content_enum::{expand_content_enum, parse::ContentEnumInput},
    event::expand_event,
    event_content::{expand_message_event_content, expand_state_event_content},
    from_raw::expand_from_raw,
    gen::RumaEvent,
    parse::RumaEventInput,
};

mod content_enum;
mod event;
mod event_content;
mod from_raw;
mod gen;
mod parse;

// A note about the `example` modules that appears in doctests:
//
// This is necessary because otherwise the expanded code appears in function context, which makes
// the compiler interpret the output of the macro as a statement, and proc macros currently aren't
// allowed to expand to statements, resulting in a compiler error.

/// Generates a Rust type for a Matrix event.
///
/// # Examples
///
/// The most common form of event is a struct with all the standard fields for an event of its
/// kind and a struct for its `content` field:
///
/// ```ignore
/// # pub mod example {
/// # use ruma_events_macros::ruma_event;
/// ruma_event! {
///     /// Informs the room about what room aliases it has been given.
///     AliasesEvent {
///         kind: StateEvent,
///         event_type: RoomAliases,
///         content: {
///             /// A list of room aliases.
///             pub aliases: Vec<ruma_identifiers::RoomAliasId>,
///         }
///     }
/// }
/// # }
/// ```
///
/// Occasionally an event will have non-standard fields at its top level (outside the `content`
/// field). These extra fields are declared in block labeled with `fields`:
///
/// ```ignore
/// # pub mod example {
/// # use ruma_events_macros::ruma_event;
/// ruma_event! {
///     /// A redaction of an event.
///     RedactionEvent {
///         kind: RoomEvent,
///         event_type: RoomRedaction,
///         fields: {
///             /// The ID of the event that was redacted.
///             pub redacts: ruma_identifiers::EventId
///         },
///         content: {
///             /// The reason for the redaction, if any.
///             pub reason: Option<String>,
///         },
///     }
/// }
/// # }
/// ```
///
/// Sometimes the type of the `content` should be a type alias rather than a struct or enum. This
/// is designated with `content_type_alias`:
///
/// ```ignore
/// # pub mod example {
/// # use ruma_events_macros::ruma_event;
/// ruma_event! {
///     /// Informs the client about the rooms that are considered direct by a user.
///     DirectEvent {
///         kind: Event,
///         event_type: Direct,
///         content_type_alias: {
///             /// The payload of a `DirectEvent`.
///             ///
///             /// A mapping of `UserId`'s to a collection of `RoomId`'s which are considered
///             /// *direct* for that particular user.
///             std::collections::BTreeMap<ruma_identifiers::UserId, Vec<ruma_identifiers::RoomId>>
///         }
///     }
/// }
/// # }
/// ```
///
/// If `content` and `content_type_alias` are both supplied, the second one listed will overwrite
/// the first.
///
/// The event type and content type will have copies generated inside a private `raw` module. These
/// "raw" versions are the same, except they implement `serde::Deserialize`. An implementation of
/// `FromRaw` will be provided, which will allow the user to deserialize the event type as
/// `EventJson<EventType>`.
#[proc_macro]
pub fn ruma_event(input: TokenStream) -> TokenStream {
    let ruma_event_input = syn::parse_macro_input!(input as RumaEventInput);

    let ruma_event = RumaEvent::from(ruma_event_input);

    ruma_event.into_token_stream().into()
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

/// Generates an implementation of `ruma_events::FromRaw`. Only usable inside of `ruma_events`.
/// Requires there to be a `raw` module in the same scope, with a type with the same name and fields
/// as the one that this macro is used on.
#[proc_macro_derive(FromRaw)]
pub fn derive_from_raw(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_from_raw(input)
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

/// Generates implementations needed to serialize and deserialize Matrix events.
#[proc_macro_derive(Event, attributes(ruma_event))]
pub fn derive_state_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
