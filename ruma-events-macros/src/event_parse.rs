//! Implementation of event enum and event content enum macros.

use std::fmt;

use matches::matches;
use proc_macro2::Span;
use quote::format_ident;
use syn::{
    parse::{self, Parse, ParseStream},
    Attribute, Expr, ExprLit, Ident, Lit, LitStr, Token,
};

/// Custom keywords for the `event_enum!` macro
mod kw {
    syn::custom_keyword!(kind);
    syn::custom_keyword!(events);
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
#[derive(Eq, PartialEq)]
pub enum EventKindVariation {
    Full,
    Sync,
    Stripped,
    Redacted,
    RedactedSync,
    RedactedStripped,
}

impl fmt::Display for EventKindVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKindVariation::Full => write!(f, ""),
            EventKindVariation::Sync => write!(f, "Sync"),
            EventKindVariation::Stripped => write!(f, "Stripped"),
            EventKindVariation::Redacted => write!(f, "Redacted"),
            EventKindVariation::RedactedSync => write!(f, "RedactedSync"),
            EventKindVariation::RedactedStripped => write!(f, "RedactedStripped"),
        }
    }
}

impl EventKindVariation {
    pub fn is_redacted(&self) -> bool {
        matches!(self, Self::Redacted | Self::RedactedSync | Self::RedactedStripped)
    }
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
pub enum EventKind {
    Basic,
    Ephemeral,
    Message,
    State,
    ToDevice,
    Redaction,
    Presence,
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKind::Basic => write!(f, "BasicEvent"),
            EventKind::Ephemeral => write!(f, "EphemeralRoomEvent"),
            EventKind::Message => write!(f, "MessageEvent"),
            EventKind::State => write!(f, "StateEvent"),
            EventKind::ToDevice => write!(f, "ToDeviceEvent"),
            EventKind::Redaction => write!(f, "RedactionEvent"),
            EventKind::Presence => write!(f, "PresenceEvent"),
        }
    }
}

impl EventKind {
    pub fn is_state(&self) -> bool {
        matches!(self, Self::State)
    }

    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message)
    }

    pub fn to_event_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        use EventKindVariation::*;

        // this match is only used to validate the input
        match (self, var) {
            (_, Full)
            | (Self::Ephemeral, Sync)
            | (Self::Message, Sync)
            | (Self::State, Sync)
            | (Self::State, Stripped)
            | (Self::Message, Redacted)
            | (Self::State, Redacted)
            | (Self::Message, RedactedSync)
            | (Self::State, RedactedSync)
            | (Self::State, RedactedStripped) => {
                Some(Ident::new(&format!("{}{}", var, self), Span::call_site()))
            }
            _ => None,
        }
    }

    pub fn to_event_enum_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        Some(format_ident!("Any{}", self.to_event_ident(var)?))
    }

    /// `Any[kind]EventContent`
    pub fn to_content_enum(&self) -> Ident {
        Ident::new(&format!("Any{}Content", self), Span::call_site())
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        Ok(match ident.to_string().as_str() {
            "Basic" => EventKind::Basic,
            "EphemeralRoom" => EventKind::Ephemeral,
            "Message" => EventKind::Message,
            "State" => EventKind::State,
            "ToDevice" => EventKind::ToDevice,
            id => {
                return Err(syn::Error::new(
                    input.span(),
                    format!(
                        "valid event kinds are Basic, EphemeralRoom, Message, State, ToDevice found `{}`",
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
        "BasicEvent" => Some((EventKind::Basic, EventKindVariation::Full)),
        "EphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Full)),
        "SyncEphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Sync)),
        "MessageEvent" => Some((EventKind::Message, EventKindVariation::Full)),
        "SyncMessageEvent" => Some((EventKind::Message, EventKindVariation::Sync)),
        "RedactedMessageEvent" => Some((EventKind::Message, EventKindVariation::Redacted)),
        "RedactedSyncMessageEvent" => Some((EventKind::Message, EventKindVariation::RedactedSync)),
        "StateEvent" => Some((EventKind::State, EventKindVariation::Full)),
        "SyncStateEvent" => Some((EventKind::State, EventKindVariation::Sync)),
        "StrippedStateEvent" => Some((EventKind::State, EventKindVariation::Stripped)),
        "RedactedStateEvent" => Some((EventKind::State, EventKindVariation::Redacted)),
        "RedactedSyncStateEvent" => Some((EventKind::State, EventKindVariation::RedactedSync)),
        "RedactedStrippedStateEvent" => {
            Some((EventKind::State, EventKindVariation::RedactedStripped))
        }
        "ToDeviceEvent" => Some((EventKind::ToDevice, EventKindVariation::Full)),
        "PresenceEvent" => Some((EventKind::Presence, EventKindVariation::Full)),
        "RedactionEvent" => Some((EventKind::Redaction, EventKindVariation::Full)),
        "SyncRedactionEvent" => Some((EventKind::Redaction, EventKindVariation::Sync)),
        _ => None,
    }
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumInput {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the event.
    pub name: EventKind,

    /// An array of valid matrix event types. This will generate the variants of the event type "name".
    /// There needs to be a corresponding variant in `ruma_events::EventType` for
    /// this event (converted to a valid Rust-style type name by stripping `m.`, replacing the
    /// remaining dots by underscores and then converting from snake_case to CamelCase).
    pub events: Vec<LitStr>,
}

impl Parse for EventEnumInput {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        // "name" field
        input.parse::<kw::kind>()?;
        input.parse::<Token![:]>()?;

        // the name of our event enum
        let name = input.parse::<EventKind>()?;
        input.parse::<Token![,]>()?;

        // "events" field
        input.parse::<kw::events>()?;
        input.parse::<Token![:]>()?;

        // an array of event names `["m.room.whatever", ...]`
        let ev_array = input.parse::<syn::ExprArray>()?;
        let events = ev_array
            .elems
            .into_iter()
            .map(|item| {
                if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = item {
                    Ok(lit_str)
                } else {
                    let msg = "values of field `events` are required to be a string literal";
                    Err(syn::Error::new_spanned(item, msg))
                }
            })
            .collect::<syn::Result<_>>()?;

        Ok(Self { attrs, name, events })
    }
}
