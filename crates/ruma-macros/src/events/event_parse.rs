//! Implementation of event enum and event content enum macros.

use std::fmt;

use proc_macro2::Span;
use quote::{format_ident, IdentFragment};
use syn::{
    braced,
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Ident, LitStr, Path, Token,
};

/// Custom keywords for the `event_enum!` macro
mod kw {
    syn::custom_keyword!(kind);
    syn::custom_keyword!(events);
    syn::custom_keyword!(alias);
    syn::custom_keyword!(ident);
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKindVariation {
    None,
    Sync,
    Original,
    OriginalSync,
    Stripped,
    Initial,
    Redacted,
    RedactedSync,
}

impl fmt::Display for EventKindVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKindVariation::None => write!(f, ""),
            EventKindVariation::Sync => write!(f, "Sync"),
            EventKindVariation::Original => write!(f, "Original"),
            EventKindVariation::OriginalSync => write!(f, "OriginalSync"),
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
        matches!(self, Self::OriginalSync | Self::RedactedSync)
    }

    pub fn to_full(self) -> Self {
        match self {
            EventKindVariation::OriginalSync => EventKindVariation::Original,
            EventKindVariation::RedactedSync => EventKindVariation::Redacted,
            _ => panic!("No original (unredacted) form of {self:?}"),
        }
    }
}

// If the variants of this enum change `to_event_path` needs to be updated as well.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    GlobalAccountData,
    RoomAccountData,
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
            EventKind::GlobalAccountData => write!(f, "GlobalAccountDataEvent"),
            EventKind::RoomAccountData => write!(f, "RoomAccountDataEvent"),
            EventKind::Ephemeral => write!(f, "EphemeralRoomEvent"),
            EventKind::MessageLike => write!(f, "MessageLikeEvent"),
            EventKind::State => write!(f, "StateEvent"),
            EventKind::ToDevice => write!(f, "ToDeviceEvent"),
            EventKind::RoomRedaction => write!(f, "RoomRedactionEvent"),
            EventKind::Presence => write!(f, "PresenceEvent"),
            EventKind::HierarchySpaceChild => write!(f, "HierarchySpaceChildEvent"),
            EventKind::Decrypted => unreachable!(),
        }
    }
}

impl IdentFragment for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl IdentFragment for EventKindVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl EventKind {
    pub fn is_account_data(self) -> bool {
        matches!(self, Self::GlobalAccountData | Self::RoomAccountData)
    }

    pub fn is_timeline(self) -> bool {
        matches!(self, Self::MessageLike | Self::RoomRedaction | Self::State)
    }

    pub fn to_event_ident(self, var: EventKindVariation) -> syn::Result<Ident> {
        use EventKindVariation as V;

        match (self, var) {
            (_, V::None)
            | (Self::Ephemeral | Self::MessageLike | Self::State, V::Sync)
            | (
                Self::MessageLike | Self::RoomRedaction | Self::State,
                V::Original | V::OriginalSync | V::Redacted | V::RedactedSync,
            )
            | (Self::State, V::Stripped | V::Initial) => Ok(format_ident!("{var}{self}")),
            _ => Err(syn::Error::new(
                Span::call_site(),
                format!("({self:?}, {var:?}) is not a valid event kind / variation combination"),
            )),
        }
    }

    pub fn to_event_enum_ident(self, var: EventKindVariation) -> syn::Result<Ident> {
        Ok(format_ident!("Any{}", self.to_event_ident(var)?))
    }

    pub fn to_event_type_enum(self) -> Ident {
        format_ident!("{}Type", self)
    }

    /// `Any[kind]EventContent`
    pub fn to_content_enum(self) -> Ident {
        format_ident!("Any{}Content", self)
    }

    /// `AnyFull[kind]EventContent`
    pub fn to_full_content_enum(self) -> Ident {
        format_ident!("AnyFull{}Content", self)
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "GlobalAccountData" => EventKind::GlobalAccountData,
            "RoomAccountData" => EventKind::RoomAccountData,
            "EphemeralRoom" => EventKind::Ephemeral,
            "MessageLike" => EventKind::MessageLike,
            "State" => EventKind::State,
            "ToDevice" => EventKind::ToDevice,
            id => {
                return Err(syn::Error::new_spanned(
                    ident,
                    format!(
                        "valid event kinds are GlobalAccountData, RoomAccountData, EphemeralRoom, \
                         MessageLike, State, ToDevice; found `{id}`",
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
        "GlobalAccountDataEvent" => Some((EventKind::GlobalAccountData, EventKindVariation::None)),
        "RoomAccountDataEvent" => Some((EventKind::RoomAccountData, EventKindVariation::None)),
        "EphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::None)),
        "SyncEphemeralRoomEvent" => Some((EventKind::Ephemeral, EventKindVariation::Sync)),
        "OriginalMessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Original)),
        "OriginalSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventKindVariation::OriginalSync))
        }
        "RedactedMessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Redacted)),
        "RedactedSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventKindVariation::RedactedSync))
        }
        "OriginalStateEvent" => Some((EventKind::State, EventKindVariation::Original)),
        "OriginalSyncStateEvent" => Some((EventKind::State, EventKindVariation::OriginalSync)),
        "StrippedStateEvent" => Some((EventKind::State, EventKindVariation::Stripped)),
        "InitialStateEvent" => Some((EventKind::State, EventKindVariation::Initial)),
        "RedactedStateEvent" => Some((EventKind::State, EventKindVariation::Redacted)),
        "RedactedSyncStateEvent" => Some((EventKind::State, EventKindVariation::RedactedSync)),
        "ToDeviceEvent" => Some((EventKind::ToDevice, EventKindVariation::None)),
        "PresenceEvent" => Some((EventKind::Presence, EventKindVariation::None)),
        "HierarchySpaceChildEvent" => {
            Some((EventKind::HierarchySpaceChild, EventKindVariation::Stripped))
        }
        "OriginalRoomRedactionEvent" => Some((EventKind::RoomRedaction, EventKindVariation::None)),
        "OriginalSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::OriginalSync))
        }
        "RedactedRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::Redacted))
        }
        "RedactedSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::RedactedSync))
        }
        "DecryptedOlmV1Event" | "DecryptedMegolmV1Event" => {
            Some((EventKind::Decrypted, EventKindVariation::None))
        }
        _ => None,
    }
}

pub struct EventEnumEntry {
    pub attrs: Vec<Attribute>,
    pub aliases: Vec<LitStr>,
    pub ev_type: LitStr,
    pub ev_path: Path,
    pub ident: Option<Ident>,
}

impl Parse for EventEnumEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (ruma_enum_attrs, attrs) = input
            .call(Attribute::parse_outer)?
            .into_iter()
            .partition::<Vec<_>, _>(|attr| attr.path().is_ident("ruma_enum"));
        let ev_type: LitStr = input.parse()?;
        let _: Token![=>] = input.parse()?;
        let ev_path = input.call(Path::parse_mod_style)?;
        let has_suffix = ev_type.value().ends_with(".*");

        let mut aliases = Vec::with_capacity(ruma_enum_attrs.len());
        let mut ident = None;

        for attr_list in ruma_enum_attrs {
            for attr in attr_list
                .parse_args_with(Punctuated::<EventEnumAttr, Token![,]>::parse_terminated)?
            {
                match attr {
                    EventEnumAttr::Alias(alias) => {
                        if alias.value().ends_with(".*") == has_suffix {
                            aliases.push(alias);
                        } else {
                            return Err(syn::Error::new_spanned(
                                &attr_list,
                                "aliases should have the same `.*` suffix, or lack thereof, as the main event type",
                            ));
                        }
                    }
                    EventEnumAttr::Ident(i) => {
                        if ident.is_some() {
                            return Err(syn::Error::new_spanned(
                                &attr_list,
                                "multiple `ident` attributes found, there can be only one",
                            ));
                        }

                        ident = Some(i);
                    }
                }
            }
        }

        Ok(Self { attrs, aliases, ev_type, ev_path, ident })
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
    /// variant in the `*EventType` enum for this event kind (converted to a valid Rust-style type
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
            let kind: EventKind = input.parse()?;

            let content;
            braced!(content in input);
            let events = content.parse_terminated(EventEnumEntry::parse, Token![,])?;
            let events = events.into_iter().collect();
            enums.push(EventEnumDecl { attrs, kind, events });
        }
        Ok(EventEnumInput { enums })
    }
}

pub enum EventEnumAttr {
    Alias(LitStr),
    Ident(Ident),
}

impl Parse for EventEnumAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            let s: LitStr = input.parse()?;
            Ok(Self::Alias(s))
        } else if lookahead.peek(kw::ident) {
            let _: kw::ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let i: Ident = input.parse()?;
            Ok(Self::Ident(i))
        } else {
            Err(lookahead.error())
        }
    }
}
