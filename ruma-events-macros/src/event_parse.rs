//! Implementation of event enum and event content enum macros.

use matches::matches;
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
    ManuallyImpled,
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
pub enum EventKind {
    Basic(Ident),
    Ephemeral(Ident),
    Message(Ident),
    State(Ident),
    ToDevice(Ident),
    ManuallyImpled(Ident),
}

impl EventKind {
    pub fn is_state(&self) -> bool {
        matches!(self, Self::State(_))
    }

    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message(_))
    }

    pub fn to_event_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        use EventKindVariation::*;

        match (self, var) {
            // all `EventKind`s are valid event structs and event enums.
            (_, Full) => Some(format_ident!("{}Event", self.get_ident())),
            (Self::Ephemeral(i), Sync) | (Self::Message(i), Sync) | (Self::State(i), Sync) => {
                Some(format_ident!("Sync{}Event", i))
            }
            (Self::State(i), Stripped) => Some(format_ident!("Stripped{}Event", i)),
            (Self::Message(i), Redacted) | (Self::State(i), Redacted) => {
                Some(format_ident!("Redacted{}Event", i))
            }
            (Self::Message(i), RedactedSync) | (Self::State(i), RedactedSync) => {
                Some(format_ident!("RedactedSync{}Event", i))
            }
            (Self::State(i), RedactedStripped) => Some(format_ident!("RedactedStripped{}Event", i)),
            _ => None,
        }
    }

    pub fn to_event_enum_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        Some(format_ident!("Any{}", self.to_event_ident(var)?))
    }

    /// `Any[kind]EventContent`
    pub fn to_content_enum(&self) -> Ident {
        format_ident!("Any{}EventContent", self.get_ident())
    }

    pub fn get_ident(&self) -> &Ident {
        match self {
            EventKind::Basic(i)
            | EventKind::Ephemeral(i)
            | EventKind::Message(i)
            | EventKind::State(i)
            | EventKind::ToDevice(i)
            | EventKind::ManuallyImpled(i) => i,
        }
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        Ok(match ident.to_string().as_str() {
            "Basic" => EventKind::Basic(ident),
            "EphemeralRoom" => EventKind::Ephemeral(ident),
            "Message" => EventKind::Message(ident),
            "State" => EventKind::State(ident),
            "ToDevice" => EventKind::ToDevice(ident),
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

pub fn to_kind_variation(ident: &Ident) -> Option<(EventKind, EventKindVariation)> {
    let ident_str = ident.to_string();
    match ident_str.as_str() {
        "BasicEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Full)),
        "EphemeralRoomEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Full)),
        "SyncEphemeralRoomEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::Sync))
        }
        "MessageEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Full)),
        "SyncMessageEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Sync)),
        "RedactedMessageEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::Redacted))
        }
        "RedactedSyncMessageEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::RedactedSync))
        }
        "StateEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Full)),
        "SyncStateEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Sync)),
        "StrippedStateEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::Stripped))
        }
        "RedactedStateEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::Redacted))
        }
        "RedactedSyncStateEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::RedactedSync))
        }
        "RedactedStrippedStateEvent" => {
            Some((EventKind::Basic(ident.clone()), EventKindVariation::RedactedStripped))
        }
        "ToDeviceEvent" => Some((EventKind::Basic(ident.clone()), EventKindVariation::Full)),
        "PresenceEvent" | "RedactionEvent" | "SyncRedactionEvent" => {
            Some((EventKind::ManuallyImpled(ident.clone()), EventKindVariation::ManuallyImpled))
        }
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
