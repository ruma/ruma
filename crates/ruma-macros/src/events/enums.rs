//! Common types for event macros.

use std::fmt;

use proc_macro2::{Span, TokenStream};
use quote::{IdentFragment, ToTokens, format_ident, quote};
use syn::{
    Ident, LitStr,
    parse::{Parse, ParseStream},
};

use crate::util::{RumaEvents, m_prefix_name_to_type_name};

/// All the possible event struct kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum EventKind {
    GlobalAccountData,
    RoomAccountData,
    EphemeralRoom,
    MessageLike,
    State,
    ToDevice,
    RoomRedaction,
    HierarchySpaceChild,
    Decrypted,
    Timeline,
}

impl EventKind {
    /// Whether this kind is account data.
    pub fn is_account_data(self) -> bool {
        matches!(self, Self::GlobalAccountData | Self::RoomAccountData)
    }

    /// Whether this kind can be found in a room's timeline.
    pub fn is_timeline(self) -> bool {
        matches!(self, Self::MessageLike | Self::RoomRedaction | Self::State | Self::Timeline)
    }

    /// Get the name of the event type (struct or enum) for this kind and the given variation.
    pub fn to_event_ident(self, var: EventVariation) -> syn::Result<Ident> {
        if !self.event_variations().contains(&var) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("({self:?}, {var:?}) is not a valid event kind / variation combination"),
            ));
        }

        Ok(format_ident!("{var}{self}"))
    }

    /// Get the name of the `Any*Event` enum for this kind and the given variation.
    pub fn to_event_enum_ident(self, var: EventVariation) -> syn::Result<Ident> {
        if !self.event_enum_variations().contains(&var) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "({self:?}, {var:?}) is not a valid event enum kind / variation combination"
                ),
            ));
        }

        Ok(format_ident!("Any{var}{self}"))
    }

    /// Get the name of the `*EventType` enum for this kind.
    pub fn to_event_type_enum(self) -> Ident {
        format_ident!("{}Type", self)
    }

    /// Get the name of the `Any[kind]EventContent` for this kind.
    pub fn to_content_enum(self) -> Ident {
        format_ident!("Any{}Content", self)
    }

    /// Get the name of the `AnyFull[kind]EventContent` for this kind.
    pub fn to_full_content_enum(self) -> Ident {
        format_ident!("AnyFull{}Content", self)
    }

    /// Get the name of the `[variation][kind]Content` trait for this kind and the given variation.
    pub fn to_content_kind_trait(self, variation: EventContentTraitVariation) -> Ident {
        format_ident!("{variation}{self}Content")
    }

    /// Get the event type (struct or enum) with its bounds for this kind and the given variation.
    pub fn to_event_with_bounds(
        self,
        var: EventVariation,
        ruma_events: &RumaEvents,
    ) -> syn::Result<EventWithBounds> {
        EventWithBounds::new(self, var, ruma_events)
    }

    /// Get the list of extra event kinds that are part of the event enum for this kind.
    pub fn extra_enum_kinds(self) -> Vec<Self> {
        match self {
            Self::MessageLike => vec![Self::RoomRedaction],
            Self::Timeline => vec![Self::MessageLike, Self::State, Self::RoomRedaction],
            Self::GlobalAccountData
            | Self::RoomAccountData
            | Self::EphemeralRoom
            | Self::State
            | Self::ToDevice
            | Self::RoomRedaction
            | Self::HierarchySpaceChild
            | Self::Decrypted => vec![],
        }
    }

    /// Get the list of variations for an event enum for this kind.
    pub fn event_enum_variations(self) -> &'static [EventVariation] {
        match self {
            Self::GlobalAccountData | Self::RoomAccountData | Self::ToDevice => {
                &[EventVariation::None]
            }
            Self::EphemeralRoom => &[EventVariation::Sync],
            Self::MessageLike | Self::Timeline => &[EventVariation::None, EventVariation::Sync],
            Self::State => &[
                EventVariation::None,
                EventVariation::Sync,
                EventVariation::Stripped,
                EventVariation::Initial,
            ],
            Self::RoomRedaction | Self::HierarchySpaceChild | Self::Decrypted => &[],
        }
    }

    /// Get the list of variations for an event type (struct or enum) for this kind.
    pub fn event_variations(self) -> &'static [EventVariation] {
        match self {
            Self::GlobalAccountData
            | Self::RoomAccountData
            | Self::ToDevice
            | Self::HierarchySpaceChild => &[EventVariation::None],
            Self::EphemeralRoom => &[EventVariation::None, EventVariation::Sync],
            Self::MessageLike | Self::RoomRedaction => &[
                EventVariation::None,
                EventVariation::Original,
                EventVariation::Redacted,
                EventVariation::Sync,
                EventVariation::OriginalSync,
                EventVariation::RedactedSync,
            ],
            Self::State => &[
                EventVariation::None,
                EventVariation::Original,
                EventVariation::Redacted,
                EventVariation::Sync,
                EventVariation::OriginalSync,
                EventVariation::RedactedSync,
                EventVariation::Stripped,
                EventVariation::Initial,
            ],
            Self::Decrypted | Self::Timeline => &[],
        }
    }

    /// Get the list of variations for an event content type for this kind.
    pub fn event_content_variations(self) -> &'static [EventContentVariation] {
        match self {
            Self::GlobalAccountData
            | Self::RoomAccountData
            | Self::EphemeralRoom
            | Self::ToDevice
            | Self::HierarchySpaceChild => &[EventContentVariation::Original],
            Self::MessageLike | Self::RoomRedaction => {
                &[EventContentVariation::Original, EventContentVariation::Redacted]
            }
            Self::State => &[
                EventContentVariation::Original,
                EventContentVariation::Redacted,
                EventContentVariation::PossiblyRedacted,
            ],
            Self::Decrypted | Self::Timeline => &[],
        }
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKind::GlobalAccountData => write!(f, "GlobalAccountDataEvent"),
            EventKind::RoomAccountData => write!(f, "RoomAccountDataEvent"),
            EventKind::EphemeralRoom => write!(f, "EphemeralRoomEvent"),
            EventKind::MessageLike => write!(f, "MessageLikeEvent"),
            EventKind::State => write!(f, "StateEvent"),
            EventKind::ToDevice => write!(f, "ToDeviceEvent"),
            EventKind::RoomRedaction => write!(f, "RoomRedactionEvent"),
            EventKind::HierarchySpaceChild => write!(f, "HierarchySpaceChildEvent"),
            EventKind::Decrypted => unreachable!(),
            EventKind::Timeline => write!(f, "TimelineEvent"),
        }
    }
}

impl IdentFragment for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "GlobalAccountData" => EventKind::GlobalAccountData,
            "RoomAccountData" => EventKind::RoomAccountData,
            "EphemeralRoom" => EventKind::EphemeralRoom,
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

/// All the possible event struct variations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventVariation {
    None,
    Sync,
    Original,
    OriginalSync,
    Stripped,
    Initial,
    Redacted,
    RedactedSync,
}

impl EventVariation {
    /// Whether this variation was redacted.
    pub fn is_redacted(self) -> bool {
        matches!(self, Self::Redacted | Self::RedactedSync)
    }

    /// Whether this variation was received via the `/sync` endpoint.
    pub fn is_sync(self) -> bool {
        matches!(self, Self::Sync | Self::OriginalSync | Self::RedactedSync)
    }

    /// Convert this "sync" variation to one which contains a `room_id`.
    ///
    /// Panics if this is not a "sync" variation.
    pub fn to_full(self) -> Self {
        match self {
            EventVariation::Sync => EventVariation::None,
            EventVariation::OriginalSync => EventVariation::Original,
            EventVariation::RedactedSync => EventVariation::Redacted,
            _ => panic!("No original (unredacted) form of {self:?}"),
        }
    }

    /// Whether this variation can implement `JsonCastable` for the other variation, if both are
    /// available for a kind.
    ///
    /// A variation can be cast to another variation when that other variation includes the same
    /// fields or less.
    pub fn is_json_castable_to(self, other: Self) -> bool {
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
            EventVariation::None => write!(f, ""),
            EventVariation::Sync => write!(f, "Sync"),
            EventVariation::Original => write!(f, "Original"),
            EventVariation::OriginalSync => write!(f, "OriginalSync"),
            EventVariation::Stripped => write!(f, "Stripped"),
            EventVariation::Initial => write!(f, "Initial"),
            EventVariation::Redacted => write!(f, "Redacted"),
            EventVariation::RedactedSync => write!(f, "RedactedSync"),
        }
    }
}

impl IdentFragment for EventVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// The possible variations of an event content type.
#[derive(Clone, Copy, PartialEq)]
pub enum EventContentVariation {
    Original,
    Redacted,
    PossiblyRedacted,
}

impl fmt::Display for EventContentVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventContentVariation::Original => Ok(()),
            EventContentVariation::Redacted => write!(f, "Redacted"),
            EventContentVariation::PossiblyRedacted => write!(f, "PossiblyRedacted"),
        }
    }
}

impl From<EventContentVariation> for EventContentTraitVariation {
    fn from(value: EventContentVariation) -> Self {
        match value {
            EventContentVariation::Original => Self::Original,
            EventContentVariation::Redacted => Self::Redacted,
            EventContentVariation::PossiblyRedacted => Self::PossiblyRedacted,
        }
    }
}

/// The possible variations of an event content trait.
#[derive(Clone, Copy, PartialEq)]
pub enum EventContentTraitVariation {
    Original,
    Redacted,
    PossiblyRedacted,
    Static,
}

impl fmt::Display for EventContentTraitVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Original => Ok(()),
            Self::Redacted => write!(f, "Redacted"),
            Self::PossiblyRedacted => write!(f, "PossiblyRedacted"),
            Self::Static => write!(f, "Static"),
        }
    }
}

/// An event type.
#[derive(Debug, Clone)]
pub struct EventType {
    source: LitStr,
    is_prefix: bool,
    value: String,
}

impl EventType {
    /// Whether this event type is a prefix.
    pub fn is_prefix(&self) -> bool {
        self.is_prefix
    }

    /// Access the inner string of this event type.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Access the inner string of this event type and remove the final `*` if this is a prefix.
    pub fn without_wildcard(&self) -> &str {
        if self.is_prefix { self.value.trim_end_matches('*') } else { &self.value }
    }

    /// Whether this event type is stable.
    ///
    /// A stable event type starts with `m.`.
    pub fn is_stable(&self) -> bool {
        self.value.starts_with("m.")
    }

    /// Get the `match` arm representation of this event type.
    pub fn as_match_arm(&self) -> TokenStream {
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

impl From<LitStr> for EventType {
    fn from(source: LitStr) -> Self {
        let value = source.value();
        Self { source, is_prefix: value.ends_with(".*"), value }
    }
}

impl Parse for EventType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(input.parse::<LitStr>()?.into())
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
pub struct EventTypes {
    pub ev_type: EventType,
    pub aliases: Vec<EventType>,
}

impl EventTypes {
    /// Try to construct an `EventTypes` from the given default event type and aliases.
    ///
    /// This performs the following validation on the event types:
    ///
    /// - `*` cannot be used anywhere in the event type but as a wildcard at the end.
    /// - If one event type ends with `.*`, all event types must end with it.
    pub fn try_from_parts(ev_type: EventType, aliases: Vec<EventType>) -> syn::Result<Self> {
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
    pub fn iter(&self) -> impl Iterator<Item = &EventType> {
        std::iter::once(&self.ev_type).chain(&self.aliases)
    }

    /// Whether the default event type is a prefix.
    ///
    /// If one event type is a prefix, all event types are prefixes.
    pub fn is_prefix(&self) -> bool {
        self.ev_type.is_prefix
    }

    /// Get the stable event type, if any.
    ///
    /// A stable type is a type beginning with `m.`.
    pub fn stable_type(&self) -> Option<&EventType> {
        self.iter().find(|ev_type| ev_type.is_stable())
    }

    /// Get the main event type.
    ///
    /// It is the stable event type or the default event type as a fallback.
    pub fn main_type(&self) -> &EventType {
        self.stable_type().unwrap_or(&self.ev_type)
    }

    /// Get the type name for these event types.
    ///
    /// Returns an error if none of these types are the stable type.
    pub fn as_event_ident(&self) -> syn::Result<Ident> {
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
pub enum EventField {
    OriginServerTs,
    RoomId,
    EventId,
    Sender,
}

impl EventField {
    /// All the variants of this enum
    pub const ALL: &[Self] = &[Self::OriginServerTs, Self::RoomId, Self::EventId, Self::Sender];

    /// Get the string representation of this field.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OriginServerTs => "origin_server_ts",
            Self::RoomId => "room_id",
            Self::EventId => "event_id",
            Self::Sender => "sender",
        }
    }

    /// This field as an [`Ident`].
    pub fn ident(self) -> Ident {
        format_ident!("{}", self.as_str())
    }

    /// Whether this field is present in the given kind and variation.
    pub fn is_present(self, kind: EventKind, var: EventVariation) -> bool {
        match self {
            Self::OriginServerTs | Self::EventId => {
                kind.is_timeline()
                    && matches!(
                        var,
                        EventVariation::None
                            | EventVariation::Sync
                            | EventVariation::Original
                            | EventVariation::OriginalSync
                            | EventVariation::Redacted
                            | EventVariation::RedactedSync
                    )
            }
            Self::RoomId => {
                matches!(
                    kind,
                    EventKind::MessageLike
                        | EventKind::State
                        | EventKind::RoomRedaction
                        | EventKind::EphemeralRoom
                ) && matches!(
                    var,
                    EventVariation::None | EventVariation::Original | EventVariation::Redacted
                )
            }
            Self::Sender => {
                matches!(
                    kind,
                    EventKind::MessageLike
                        | EventKind::State
                        | EventKind::RoomRedaction
                        | EventKind::ToDevice
                ) && var != EventVariation::Initial
            }
        }
    }

    /// Get the type of this field.
    ///
    /// Returns a `(type, is_reference)` tuple.
    pub fn ty(self, ruma_events: &RumaEvents) -> (TokenStream, bool) {
        let ruma_common = ruma_events.ruma_common();

        match self {
            Self::OriginServerTs => (quote! { #ruma_common::MilliSecondsSinceUnixEpoch }, false),
            Self::RoomId => (quote! { &#ruma_common::RoomId }, true),
            Self::EventId => (quote! { &#ruma_common::EventId }, true),
            Self::Sender => (quote! { &#ruma_common::UserId }, true),
        }
    }
}

impl fmt::Display for EventField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An event type (struct or enum) with its bounds.
pub struct EventWithBounds {
    pub type_with_generics: TokenStream,
    pub impl_generics: Option<TokenStream>,
    pub where_clause: Option<TokenStream>,
}

impl EventWithBounds {
    pub fn new(
        kind: EventKind,
        var: EventVariation,
        ruma_events: &RumaEvents,
    ) -> syn::Result<Self> {
        let ident = kind.to_event_ident(var)?;

        let event_content_trait = match var {
            EventVariation::None
            | EventVariation::Sync
            | EventVariation::Original
            | EventVariation::OriginalSync
            | EventVariation::Initial => {
                // `State` event structs have a `StaticStateEventContent` bound.
                if kind == EventKind::State {
                    kind.to_content_kind_trait(EventContentTraitVariation::Static)
                } else {
                    kind.to_content_kind_trait(EventContentTraitVariation::Original)
                }
            }
            EventVariation::Stripped => {
                kind.to_content_kind_trait(EventContentTraitVariation::PossiblyRedacted)
            }
            EventVariation::Redacted | EventVariation::RedactedSync => {
                kind.to_content_kind_trait(EventContentTraitVariation::Redacted)
            }
        };

        let (type_with_generics, impl_generics, where_clause) = match kind {
            EventKind::MessageLike | EventKind::State
                if matches!(var, EventVariation::None | EventVariation::Sync) =>
            {
                // `MessageLike` and `State` event kinds have an extra `RedactContent` bound with a
                // `where` clause on the variations that match enum types.
                let redacted_trait =
                    kind.to_content_kind_trait(EventContentTraitVariation::Redacted);

                (
                    quote! { #ruma_events::#ident<C> },
                    Some(
                        quote! { <C: #ruma_events::#event_content_trait + #ruma_events::RedactContent> },
                    ),
                    Some(quote! {
                        where
                            C::Redacted: #ruma_events::#redacted_trait,
                    }),
                )
            }
            EventKind::GlobalAccountData
            | EventKind::RoomAccountData
            | EventKind::EphemeralRoom
            | EventKind::MessageLike
            | EventKind::State
            | EventKind::ToDevice => (
                quote! { #ruma_events::#ident<C> },
                Some(quote! { <C: #ruma_events::#event_content_trait> }),
                None,
            ),
            EventKind::RoomRedaction => {
                (quote! { #ruma_events::room::redaction::#ident }, None, None)
            }
            // These don't have an event type and will fail in the `to_event_ident()` call above.
            EventKind::HierarchySpaceChild | EventKind::Decrypted | EventKind::Timeline => {
                unreachable!()
            }
        };

        Ok(Self { impl_generics, type_with_generics, where_clause })
    }
}
