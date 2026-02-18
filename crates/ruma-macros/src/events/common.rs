//! Common types for event macros.

use std::fmt;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};

use crate::util::{RumaEvents, m_prefix_name_to_type_name};

/// All the common event kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(super) enum CommonEventKind {
    /// Global account data.
    ///
    /// This is user data for the whole account.
    GlobalAccountData,

    /// Room account data.
    ///
    /// This is user data specific to a room.
    RoomAccountData,

    /// Ephemeral room data.
    ///
    /// This is data associated to a room and that is not persisted.
    EphemeralRoom,

    /// Message-like event.
    ///
    /// This is an event that can occur in the timeline and that doesn't have a state key.
    MessageLike,

    /// State event.
    ///
    /// This is an event that can occur in the timeline and that has a state key.
    State,

    /// A to-device event.
    ///
    /// This is an event that is sent directly to another device.
    ToDevice,
}

impl CommonEventKind {
    /// Get the list of variations for an event type (struct or enum) for this kind.
    pub(super) fn event_variations(self) -> &'static [EventVariation] {
        match self {
            Self::GlobalAccountData | Self::RoomAccountData | Self::ToDevice => {
                &[EventVariation::None]
            }
            Self::EphemeralRoom => &[EventVariation::None, EventVariation::Sync],
            Self::MessageLike => &[
                EventVariation::None,
                EventVariation::Original,
                EventVariation::Redacted,
                EventVariation::Sync,
                EventVariation::OriginalSync,
                EventVariation::RedactedSync,
            ],
            Self::State => &[
                EventVariation::None,
                EventVariation::Sync,
                EventVariation::Stripped,
                EventVariation::Initial,
            ],
        }
    }
}

impl fmt::Display for CommonEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GlobalAccountData => write!(f, "GlobalAccountDataEvent"),
            Self::RoomAccountData => write!(f, "RoomAccountDataEvent"),
            Self::EphemeralRoom => write!(f, "EphemeralRoomEvent"),
            Self::MessageLike => write!(f, "MessageLikeEvent"),
            Self::State => write!(f, "StateEvent"),
            Self::ToDevice => write!(f, "ToDeviceEvent"),
        }
    }
}

impl Parse for CommonEventKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "GlobalAccountData" => Self::GlobalAccountData,
            "RoomAccountData" => Self::RoomAccountData,
            "EphemeralRoom" => Self::EphemeralRoom,
            "MessageLike" => Self::MessageLike,
            "State" => Self::State,
            "ToDevice" => Self::ToDevice,
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

/// All the possible event variations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EventVariation {
    /// The full format of an event.
    ///
    /// Either the event cannot be redacted, or the type contains variants for the original and
    /// redacted variations.
    None,

    /// The sync format of an event.
    ///
    /// Either the event cannot be redacted, or the type contains variants for the original and
    /// redacted variations.
    Sync,

    /// The full format of an event that can be redacted.
    Original,

    /// The sync format of an event that can be redacted.
    OriginalSync,

    /// The stripped format of an event.
    Stripped,

    /// The format of an event passed during room creation.
    Initial,

    /// The full format of an event that was redacted.
    Redacted,

    /// The sync format of an event that was redacted.
    RedactedSync,
}

impl EventVariation {
    /// Whether this variation was redacted.
    pub(super) fn is_redacted(self) -> bool {
        matches!(self, Self::Redacted | Self::RedactedSync)
    }

    /// Whether this variation was received via the `/sync` endpoint.
    pub(super) fn is_sync(self) -> bool {
        matches!(self, Self::Sync | Self::OriginalSync | Self::RedactedSync)
    }

    /// Convert this "sync" variation to one which contains a `room_id`, if possible.
    ///
    /// Returns `None` if this is not a "sync" variation.
    pub(super) fn to_full(self) -> Option<Self> {
        Some(match self {
            Self::Sync => Self::None,
            Self::OriginalSync => Self::Original,
            Self::RedactedSync => Self::Redacted,
            _ => return None,
        })
    }

    /// Whether this variation can implement `JsonCastable` for the other variation, if both are
    /// available for a kind.
    ///
    /// A variation can be cast to another variation when that other variation includes the same
    /// fields or less.
    pub(super) fn is_json_castable_to(self, other: Self) -> bool {
        match self {
            Self::None | Self::OriginalSync | Self::RedactedSync => {
                matches!(other, Self::Sync | Self::Stripped)
            }
            Self::Original => {
                matches!(other, Self::None | Self::Sync | Self::OriginalSync | Self::Stripped)
            }
            Self::Redacted => {
                matches!(other, Self::None | Self::Sync | Self::RedactedSync | Self::Stripped)
            }
            Self::Sync | Self::Stripped | Self::Initial => false,
        }
    }
}

impl fmt::Display for EventVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Sync => write!(f, "Sync"),
            Self::Original => write!(f, "Original"),
            Self::OriginalSync => write!(f, "OriginalSync"),
            Self::Stripped => write!(f, "Stripped"),
            Self::Initial => write!(f, "Initial"),
            Self::Redacted => write!(f, "Redacted"),
            Self::RedactedSync => write!(f, "RedactedSync"),
        }
    }
}

/// The possible variations of an event content trait.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum EventContentTraitVariation {
    /// An event content that wasn't redacted.
    Original,

    /// An event content that was redacted.
    Redacted,

    /// Static data about an event content that wasn't redacted.
    Static,
}

impl fmt::Display for EventContentTraitVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Original => Ok(()),
            Self::Redacted => write!(f, "Redacted"),
            Self::Static => write!(f, "Static"),
        }
    }
}

/// An event type.
#[derive(Debug, Clone)]
pub(super) struct EventType {
    /// The source of the event type.
    source: syn::LitStr,

    /// Whether this event type is a prefix.
    is_prefix: bool,

    /// The value of the event type.
    value: String,
}

impl EventType {
    /// Whether this event type is a prefix.
    pub(super) fn is_prefix(&self) -> bool {
        self.is_prefix
    }

    /// Access the inner string of this event type.
    pub(super) fn as_str(&self) -> &str {
        &self.value
    }

    /// Access the inner string of this event type and remove the final `*` if this is a prefix.
    pub(super) fn without_wildcard(&self) -> &str {
        if self.is_prefix { self.value.trim_end_matches('*') } else { &self.value }
    }

    /// Whether this event type is stable.
    ///
    /// A stable event type starts with `m.`.
    pub(super) fn is_stable(&self) -> bool {
        self.value.starts_with("m.")
    }

    /// Get the `match` arm representation of this event type.
    pub(super) fn as_match_arm(&self) -> TokenStream {
        let ev_type = self.without_wildcard();

        if self.is_prefix() {
            quote! { t if t.starts_with(#ev_type) }
        } else {
            quote! { #ev_type }
        }
    }
}

impl PartialEq for EventType {
    fn eq(&self, other: &Self) -> bool {
        self.is_prefix == other.is_prefix && self.value == other.value
    }
}

impl Eq for EventType {}

impl From<syn::LitStr> for EventType {
    fn from(source: syn::LitStr) -> Self {
        let value = source.value();
        Self { source, is_prefix: value.ends_with(".*"), value }
    }
}

impl Parse for EventType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(input.parse::<syn::LitStr>()?.into())
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ToTokens for EventType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.source.to_tokens(tokens);
    }
}

/// All the event types supported by an event.
#[derive(Clone)]
pub(super) struct EventTypes {
    /// The main event type.
    pub(super) ev_type: EventType,

    /// The alternate event types.
    pub(super) aliases: Vec<EventType>,
}

impl EventTypes {
    /// Try to construct an `EventTypes` from the given default event type and aliases.
    ///
    /// This performs the following validation on the event types:
    ///
    /// - `*` cannot be used anywhere in the event type but as a wildcard at the end.
    /// - If one event type ends with `.*`, all event types must end with it.
    pub(super) fn try_from_parts(ev_type: EventType, aliases: Vec<EventType>) -> syn::Result<Self> {
        if ev_type.without_wildcard().contains('*') {
            return Err(syn::Error::new_spanned(
                ev_type,
                "event type may only contain `*` as part of a `.*` suffix",
            ));
        }

        let is_prefix = ev_type.is_prefix();

        for alias in &aliases {
            if alias.without_wildcard().contains('*') {
                return Err(syn::Error::new_spanned(
                    alias,
                    "alias may only contain `*` as part of a `.*` suffix",
                ));
            }

            if alias.is_prefix() != is_prefix {
                return Err(syn::Error::new_spanned(
                    alias,
                    "aliases should have the same `.*` suffix, or lack thereof, as the main event type",
                ));
            }
        }

        Ok(Self { ev_type, aliases })
    }

    /// Get an iterator over all the event types.
    pub(super) fn iter(&self) -> impl Iterator<Item = &EventType> {
        std::iter::once(&self.ev_type).chain(&self.aliases)
    }

    /// Whether the default event type is a prefix.
    ///
    /// If one event type is a prefix, all event types are prefixes.
    pub(super) fn is_prefix(&self) -> bool {
        self.ev_type.is_prefix
    }

    /// Get the stable event type, if any.
    ///
    /// A stable type is a type beginning with `m.`.
    pub(super) fn stable_type(&self) -> Option<&EventType> {
        self.iter().find(|ev_type| ev_type.is_stable())
    }

    /// Get the main event type.
    ///
    /// It is the stable event type or the default event type as a fallback.
    pub(super) fn main_type(&self) -> &EventType {
        self.stable_type().unwrap_or(&self.ev_type)
    }

    /// Get the type name for these event types.
    ///
    /// Returns an error if none of these types are the stable type.
    pub(super) fn as_event_ident(&self) -> syn::Result<syn::Ident> {
        let stable_type = self.stable_type().ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                format!(
                    "A matrix event must declare a well-known type that starts with `m.` \
                     either as the main type or as an alias, or must declare the ident that \
                     should be used if it is only an unstable type, found main type `{}`",
                    self.ev_type
                ),
            )
        })?;

        Ok(m_prefix_name_to_type_name(&stable_type.source)
            .expect("we already checked that the event type is stable"))
    }
}

/// Common fields in event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CommonEventField {
    /// `origin_server_ts`.
    OriginServerTs,

    /// `room_id`.
    RoomId,

    /// `event_id`.
    EventId,

    /// `sender`.
    Sender,
}

impl CommonEventField {
    /// All the variants of this enum
    pub(super) const ALL: &[Self] =
        &[Self::OriginServerTs, Self::RoomId, Self::EventId, Self::Sender];

    /// Get the string representation of this field.
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::OriginServerTs => "origin_server_ts",
            Self::RoomId => "room_id",
            Self::EventId => "event_id",
            Self::Sender => "sender",
        }
    }

    /// This field as a [`syn::Ident`].
    pub(super) fn ident(self) -> syn::Ident {
        format_ident!("{}", self.as_str())
    }

    /// Get the type of this field.
    ///
    /// Returns a `(type, is_reference)` tuple.
    pub(super) fn ty(self, ruma_events: &RumaEvents) -> (TokenStream, bool) {
        let ruma_common = ruma_events.ruma_common();

        match self {
            Self::OriginServerTs => (quote! { #ruma_common::MilliSecondsSinceUnixEpoch }, false),
            Self::RoomId => (quote! { &#ruma_common::RoomId }, true),
            Self::EventId => (quote! { &#ruma_common::EventId }, true),
            Self::Sender => (quote! { &#ruma_common::UserId }, true),
        }
    }
}

impl fmt::Display for CommonEventField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
