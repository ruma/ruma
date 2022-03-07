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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    pub fn is_sync(self) -> bool {
        matches!(self, Self::Sync | Self::RedactedSync)
    }

    pub fn to_redacted(self) -> Self {
        match self {
            EventKindVariation::Full => EventKindVariation::Redacted,
            EventKindVariation::Sync => EventKindVariation::RedactedSync,
            _ => panic!("No redacted form of {:?}", self),
        }
    }

    pub fn to_sync(self) -> Self {
        match self {
            EventKindVariation::Full => EventKindVariation::Sync,
            EventKindVariation::Redacted => EventKindVariation::RedactedSync,
            _ => panic!("No sync form of {:?}", self),
        }
    }

    pub fn to_full(self) -> Self {
        match self {
            EventKindVariation::Sync => EventKindVariation::Full,
            EventKindVariation::RedactedSync => EventKindVariation::Redacted,
            _ => panic!("No full form of {:?}", self),
        }
    }
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    Ephemeral,
    MessageLike,
    State,
    ToDevice,
    RoomRedaction,
    Presence,
    HierarchySpaceChild,
    Decrypted,
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKind::Ephemeral => write!(f, "EphemeralRoomEvent"),
            EventKind::MessageLike => write!(f, "MessageLikeEvent"),
            EventKind::State => write!(f, "StateEvent"),
            EventKind::ToDevice => write!(f, "ToDeviceEvent"),
            EventKind::RoomRedaction => write!(f, "RoomRedactionEvent"),
            EventKind::Presence => write!(f, "PresenceEvent"),
            EventKind::HierarchySpaceChild => write!(f, "HierarchySpaceChildStateEvent"),
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
    pub fn try_to_event_ident(self, var: EventKindVariation) -> Option<Ident> {
        use EventKindVariation as V;

        match (self, var) {
            (_, V::Full)
            | (Self::MessageLike | Self::RoomRedaction | Self::State | Self::Ephemeral, V::Sync)
            | (
                Self::MessageLike | Self::RoomRedaction | Self::State,
                V::Redacted | V::RedactedSync,
            )
            | (Self::State, V::Stripped | V::Initial) => Some(format_ident!("{}{}", var, self)),
            _ => None,
        }
    }

    pub fn to_event_ident(self, var: EventKindVariation) -> Ident {
        self.try_to_event_ident(var).unwrap_or_else(|| {
            panic!("({:?}, {:?}) is not a valid event kind / variation combination", self, var);
        })
    }

    pub fn to_event_enum_ident(self, var: EventKindVariation) -> Ident {
        format_ident!("Any{}", self.to_event_ident(var))
    }

    /// `Any[kind]EventContent`
    pub fn to_content_enum(self) -> Ident {
        format_ident!("Any{}Content", self)
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "EphemeralRoom" => EventKind::Ephemeral,
            "MessageLike" => EventKind::MessageLike,
            "State" => EventKind::State,
            "ToDevice" => EventKind::ToDevice,
            id => {
                return Err(syn::Error::new_spanned(
                    ident,
                    format!(
                        "valid event kinds are GlobalAccountData, RoomAccountData, EphemeralRoom, \
                        MessageLike, State, ToDevice found `{}`",
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
        "EphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Full)),
        "SyncEphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Sync)),
        "MessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Full)),
        "SyncMessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Sync)),
        "RedactedMessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Redacted)),
        "RedactedSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventKindVariation::RedactedSync))
        }
        "StateEvent" => Some((EventKind::State, EventKindVariation::Full)),
        "SyncStateEvent" => Some((EventKind::State, EventKindVariation::Sync)),
        "StrippedStateEvent" => Some((EventKind::State, EventKindVariation::Stripped)),
        "InitialStateEvent" => Some((EventKind::State, EventKindVariation::Initial)),
        "RedactedStateEvent" => Some((EventKind::State, EventKindVariation::Redacted)),
        "RedactedSyncStateEvent" => Some((EventKind::State, EventKindVariation::RedactedSync)),
        "ToDeviceEvent" => Some((EventKind::ToDevice, EventKindVariation::Full)),
        "PresenceEvent" => Some((EventKind::Presence, EventKindVariation::Full)),
        "HierarchySpaceChildStateEvent" => {
            Some((EventKind::HierarchySpaceChild, EventKindVariation::Stripped))
        }
        "RoomRedactionEvent" => Some((EventKind::RoomRedaction, EventKindVariation::Full)),
        "SyncRoomRedactionEvent" => Some((EventKind::RoomRedaction, EventKindVariation::Sync)),
        "RedactedRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::Redacted))
        }
        "RedactedSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::RedactedSync))
        }
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

    /// The event kind.
    pub kind: EventKind,

    /// An array of valid matrix event types.
    ///
    /// This will generate the variants of the event type "kind". There needs to be a corresponding
    /// variant in `ruma_common::events::EventType` for this event (converted to a valid Rust-style
    /// type name by stripping `m.`, replacing the remaining dots by underscores and then
    /// converting from snake_case to CamelCase).
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
            let kind: EventKind = input.parse()?;

            let content;
            braced!(content in input);
            let events = content.parse_terminated::<_, Token![,]>(EventEnumEntry::parse)?;
            let events = events.into_iter().collect();
            enums.push(EventEnumDecl { attrs, kind, events });
        }
        Ok(EventEnumInput { enums })
    }
}
