//! Common types for event macros.

use std::fmt;

use proc_macro2::Span;
use quote::{format_ident, IdentFragment};
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

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
}

impl EventKind {
    /// Whether this kind is account data.
    pub fn is_account_data(self) -> bool {
        matches!(self, Self::GlobalAccountData | Self::RoomAccountData)
    }

    /// Whether this kind can be found in a room's timeline.
    pub fn is_timeline(self) -> bool {
        matches!(self, Self::MessageLike | Self::RoomRedaction | Self::State)
    }

    /// Get the name of the event struct for this kind and the given variation.
    pub fn to_event_ident(self, var: EventKindVariation) -> syn::Result<Ident> {
        use EventKindVariation as V;

        match (self, var) {
            (_, V::None)
            | (Self::EphemeralRoom | Self::MessageLike | Self::State, V::Sync)
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

    /// Get the name of the `Any*Event` enum for this kind and the given variation.
    pub fn to_event_enum_ident(self, var: EventKindVariation) -> syn::Result<Ident> {
        Ok(format_ident!("Any{}", self.to_event_ident(var)?))
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
    pub fn to_content_kind_trait(self, variation: EventKindContentVariation) -> Ident {
        format_ident!("{variation}{self}Content")
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

impl EventKindVariation {
    /// Whether this variation can contain an `event_id` (depending on the kind).
    pub fn has_event_id(self) -> bool {
        matches!(
            self,
            Self::None
                | Self::Sync
                | Self::Original
                | Self::OriginalSync
                | Self::Redacted
                | Self::RedactedSync
        )
    }

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
            EventKindVariation::Sync => EventKindVariation::None,
            EventKindVariation::OriginalSync => EventKindVariation::Original,
            EventKindVariation::RedactedSync => EventKindVariation::Redacted,
            _ => panic!("No original (unredacted) form of {self:?}"),
        }
    }
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

impl IdentFragment for EventKindVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// The possible variations of an event content type.
#[derive(Clone, Copy, PartialEq)]
pub enum EventKindContentVariation {
    Original,
    Redacted,
    PossiblyRedacted,
}

impl fmt::Display for EventKindContentVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKindContentVariation::Original => Ok(()),
            EventKindContentVariation::Redacted => write!(f, "Redacted"),
            EventKindContentVariation::PossiblyRedacted => write!(f, "PossiblyRedacted"),
        }
    }
}
