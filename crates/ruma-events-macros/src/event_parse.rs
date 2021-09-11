//! Implementation of event enum and event content enum macros.

use std::fmt;

use proc_macro2::Span;
use quote::{format_ident, IdentFragment};
use syn::{
    braced,
    parse::{self, Parse, ParseStream},
    Attribute, Ident, LitStr, Token,
};

/// Custom keywords for the `event_enum!` macro
mod kw {
    syn::custom_keyword!(kind);
    syn::custom_keyword!(events);
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EventKindVariation {
    Full,
    Sync,
    Stripped,
    Initial,
    Redacted,
    RedactedSync,
}

impl fmt::Display for EventKindVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKindVariation::Full => write!(f, ""),
            EventKindVariation::Sync => write!(f, "Sync"),
            EventKindVariation::Stripped => write!(f, "Stripped"),
            EventKindVariation::Initial => write!(f, "Initial"),
            EventKindVariation::Redacted => write!(f, "Redacted"),
            EventKindVariation::RedactedSync => write!(f, "RedactedSync"),
        }
    }
}

impl EventKindVariation {
    pub fn is_redacted(self) -> bool {
        matches!(self, Self::Redacted | Self::RedactedSync)
    }

    pub fn to_full_variation(self) -> Self {
        match self {
            EventKindVariation::Redacted | EventKindVariation::RedactedSync => {
                EventKindVariation::Redacted
            }
            EventKindVariation::Full
            | EventKindVariation::Sync
            | EventKindVariation::Stripped
            | EventKindVariation::Initial => EventKindVariation::Full,
        }
    }
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
#[derive(Debug, Eq, PartialEq)]
pub enum EventKind {
    GlobalAccountData,
    RoomAccountData,
    Ephemeral,
    Message,
    State,
    ToDevice,
    Redaction,
    Presence,
    Decrypted,
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKind::GlobalAccountData => write!(f, "GlobalAccountDataEvent"),
            EventKind::RoomAccountData => write!(f, "RoomAccountDataEvent"),
            EventKind::Ephemeral => write!(f, "EphemeralRoomEvent"),
            EventKind::Message => write!(f, "MessageEvent"),
            EventKind::State => write!(f, "StateEvent"),
            EventKind::ToDevice => write!(f, "ToDeviceEvent"),
            EventKind::Redaction => write!(f, "RedactionEvent"),
            EventKind::Presence => write!(f, "PresenceEvent"),
            EventKind::Decrypted => unreachable!(),
        }
    }
}

impl IdentFragment for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }

    fn span(&self) -> Option<Span> {
        Some(Span::call_site())
    }
}

impl IdentFragment for EventKindVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }

    fn span(&self) -> Option<Span> {
        Some(Span::call_site())
    }
}

impl EventKind {
    pub fn to_event_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        use EventKindVariation as V;

        // this match is only used to validate the input
        match (self, var) {
            (_, V::Full)
            | (Self::Ephemeral, V::Sync)
            | (Self::Message, V::Sync)
            | (Self::State, V::Sync)
            | (Self::State, V::Stripped)
            | (Self::State, V::Initial)
            | (Self::Message, V::Redacted)
            | (Self::State, V::Redacted)
            | (Self::Message, V::RedactedSync)
            | (Self::State, V::RedactedSync) => Some(format_ident!("{}{}", var, self)),
            _ => None,
        }
    }

    pub fn to_event_enum_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        Some(format_ident!("Any{}", self.to_event_ident(var)?))
    }

    /// `Any[kind]EventContent`
    pub fn to_content_enum(&self) -> Ident {
        format_ident!("Any{}Content", self)
    }

    /// `AnyRedacted[kind]EventContent`
    pub fn to_redacted_content_enum(&self) -> Ident {
        format_ident!("AnyRedacted{}Content", self)
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "GlobalAccountData" => EventKind::GlobalAccountData,
            "RoomAccountData" => EventKind::RoomAccountData,
            "EphemeralRoom" => EventKind::Ephemeral,
            "Message" => EventKind::Message,
            "State" => EventKind::State,
            "ToDevice" => EventKind::ToDevice,
            id => {
                return Err(syn::Error::new_spanned(
                    ident,
                    format!(
                        "valid event kinds are GlobalAccountData, RoomAccountData, EphemeralRoom, \
                        Message, State, ToDevice found `{}`",
                        id
                    ),
                ));
            }
        })
    }
}

// This function is only used in the `Event` derive macro expansion code.
/// Validates the given `ident` is a valid event struct name and returns a tuple of enums
/// representing the name.
pub fn to_kind_variation(ident: &Ident) -> Option<(EventKind, EventKindVariation)> {
    let ident_str = ident.to_string();
    match ident_str.as_str() {
        "GlobalAccountDataEvent" => Some((EventKind::GlobalAccountData, EventKindVariation::Full)),
        "RoomAccountDataEvent" => Some((EventKind::RoomAccountData, EventKindVariation::Full)),
        "EphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Full)),
        "SyncEphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Sync)),
        "MessageEvent" => Some((EventKind::Message, EventKindVariation::Full)),
        "SyncMessageEvent" => Some((EventKind::Message, EventKindVariation::Sync)),
        "RedactedMessageEvent" => Some((EventKind::Message, EventKindVariation::Redacted)),
        "RedactedSyncMessageEvent" => Some((EventKind::Message, EventKindVariation::RedactedSync)),
        "StateEvent" => Some((EventKind::State, EventKindVariation::Full)),
        "SyncStateEvent" => Some((EventKind::State, EventKindVariation::Sync)),
        "StrippedStateEvent" => Some((EventKind::State, EventKindVariation::Stripped)),
        "InitialStateEvent" => Some((EventKind::State, EventKindVariation::Initial)),
        "RedactedStateEvent" => Some((EventKind::State, EventKindVariation::Redacted)),
        "RedactedSyncStateEvent" => Some((EventKind::State, EventKindVariation::RedactedSync)),
        "ToDeviceEvent" => Some((EventKind::ToDevice, EventKindVariation::Full)),
        "PresenceEvent" => Some((EventKind::Presence, EventKindVariation::Full)),
        "RedactionEvent" => Some((EventKind::Redaction, EventKindVariation::Full)),
        "SyncRedactionEvent" => Some((EventKind::Redaction, EventKindVariation::Sync)),
        "DecryptedOlmV1Event" | "DecryptedMegolmV1Event" => {
            Some((EventKind::Decrypted, EventKindVariation::Full))
        }
        _ => None,
    }
}

pub struct EventEnumEntry {
    pub attrs: Vec<Attribute>,
    pub ev_type: LitStr,
}

impl Parse for EventEnumEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self { attrs: input.call(Attribute::parse_outer)?, ev_type: input.parse()? })
    }
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumDecl {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the event.
    pub name: EventKind,

    /// An array of valid matrix event types.
    ///
    /// This will generate the variants of the event type "kind". There needs to be a corresponding
    /// variant in `ruma_events::EventType` for this event (converted to a valid Rust-style type
    /// name by stripping `m.`, replacing the remaining dots by underscores and then converting
    /// from snake_case to CamelCase).
    pub events: Vec<EventEnumEntry>,
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumInput {
    pub(crate) enums: Vec<EventEnumDecl>,
}

impl Parse for EventEnumInput {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let mut enums = vec![];
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;

            let _: Token![enum] = input.parse()?;
            let name: EventKind = input.parse()?;

            let content;
            braced!(content in input);
            let events = content.parse_terminated::<_, Token![,]>(EventEnumEntry::parse)?;
            let events = events.into_iter().collect();
            enums.push(EventEnumDecl { attrs, name, events });
        }
        Ok(EventEnumInput { enums })
    }
}
