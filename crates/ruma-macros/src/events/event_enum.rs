//! Implementation of the `event_enum!` macro.

use std::fmt;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod event_kind_enum;
mod event_type;
mod parse;
mod util;

use self::{
    event_kind_enum::EventEnum, event_type::EventTypeEnum, util::expand_json_castable_impl,
};
use super::common::{
    CommonEventField, CommonEventKind, EventContentTraitVariation, EventTypes, EventVariation,
};
use crate::util::RumaEvents;

/// Generates enums to represent the various Matrix event types.
pub(crate) fn expand_event_enum(input: EventEnumInput) -> TokenStream {
    let ruma_events = RumaEvents::new();
    let mut event_enums_data = input.enums;
    let mut tokens = TokenStream::new();
    let mut timeline_data = None;

    for data in &event_enums_data {
        tokens.extend(
            EventEnum::new(data, &ruma_events)
                .expand()
                .unwrap_or_else(syn::Error::into_compile_error),
        );

        // Create the Timeline kind if there are events to put in it. The `Any*TimelineEvent` enums
        // are implemented manually so we don't need to generate them.
        if data.kind.is_timeline() {
            timeline_data
                .get_or_insert_with(|| EventEnumData {
                    attrs: Vec::new(),
                    kind: EventEnumKind::Timeline,
                    events: Vec::new(),
                })
                .events
                .extend(data.events.iter().cloned());
        }
    }

    // Handle the Timeline kind if necessary.
    if let Some(mut data) = timeline_data {
        // Deduplicate event variants, in case there are some with the same `type` in the timeline
        // kinds. This is necessary for the `m.room.encrypted` state event type from MSC4362.
        let mut deduped_events: Vec<EventEnumEntry> = Vec::new();
        for event in data.events {
            if let Some(idx) = deduped_events
                .iter()
                .position(|deduped_event| deduped_event.types.ev_type == event.types.ev_type)
            {
                // If there is a variant without config attributes use that.
                if deduped_events[idx].attrs != event.attrs && event.attrs.is_empty() {
                    deduped_events[idx] = event;
                }
            } else {
                deduped_events.push(event);
            }
        }
        data.events = deduped_events;

        // Generate `JsonCastable` implementations for `Any*TimelineEvent` enums.
        tokens.extend(data.kind.event_enum_variations().iter().map(|variation| {
            let ident = data.kind.to_event_enum_ident(*variation);
            expand_json_castable_impl(&ident, data.kind, *variation, &ruma_events)
        }));

        event_enums_data.push(data);
    }

    tokens.extend(
        event_enums_data.iter().map(|data| EventTypeEnum::new(data, &ruma_events).expand()),
    );

    tokens
}

/// The parsed `event_enum!` macro.
pub(crate) struct EventEnumInput {
    /// The parsed enums.
    enums: Vec<EventEnumData>,
}

/// The parsed data for a specific [`EventKind`] in the `event_enum!` macro.
struct EventEnumData {
    /// Outer attributes on the declaration, such as docstrings.
    attrs: Vec<syn::Attribute>,

    /// The event enum kind.
    kind: EventEnumKind,

    /// The event types for this kind.
    events: Vec<EventEnumEntry>,
}

/// All the possible [`EventEnum`] kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventEnumKind {
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

    /// Timeline event.
    ///
    /// This is any event that can occur in the timeline, so this includes message-like and state
    /// events.
    Timeline,

    /// A to-device event.
    ///
    /// This is an event that is sent directly to another device.
    ToDevice,
}

impl EventEnumKind {
    /// Whether this kind can be found in a room's timeline.
    fn is_timeline(self) -> bool {
        matches!(self, Self::MessageLike | Self::State)
    }

    /// The common kind matching this kind, if any.
    ///
    /// Returns `None` for the [`EventEnumKind::Timeline`] variant.
    fn common_kind(self) -> Option<CommonEventKind> {
        Some(match self {
            Self::GlobalAccountData => CommonEventKind::GlobalAccountData,
            Self::RoomAccountData => CommonEventKind::RoomAccountData,
            Self::EphemeralRoom => CommonEventKind::EphemeralRoom,
            Self::MessageLike => CommonEventKind::MessageLike,
            Self::State => CommonEventKind::State,
            Self::ToDevice => CommonEventKind::ToDevice,
            Self::Timeline => return None,
        })
    }

    /// Get the name of the event type (struct or enum) for this kind and the given variation.
    fn to_event_ident(self, variation: EventVariation) -> syn::Ident {
        format_ident!("{variation}{self}")
    }

    /// Get the name of the `*EventType` enum for this kind.
    fn to_event_type_enum(self) -> syn::Ident {
        format_ident!("{self}Type")
    }

    /// Get the name of the `{variation}{kind}Content` trait for this kind and the given variation.
    fn to_content_kind_trait(self, variation: EventContentTraitVariation) -> syn::Ident {
        format_ident!("{variation}{self}Content")
    }

    /// Get the list of variations for an event type (struct or enum) for this kind.
    fn event_variations(self) -> &'static [EventVariation] {
        if let Some(common_kind) = self.common_kind() {
            common_kind.event_variations()
        } else {
            // The Timeline kind has no variations.
            &[]
        }
    }

    /// Get the list of variations for an event enum for this kind.
    fn event_enum_variations(self) -> &'static [EventVariation] {
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
        }
    }

    /// Whether the given field is present in this kind and variation.
    fn field_is_present(self, field: CommonEventField, var: EventVariation) -> bool {
        match field {
            CommonEventField::OriginServerTs | CommonEventField::EventId => {
                self.is_timeline()
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
            CommonEventField::RoomId => {
                matches!(self, Self::MessageLike | Self::State | Self::EphemeralRoom)
                    && matches!(
                        var,
                        EventVariation::None | EventVariation::Original | EventVariation::Redacted
                    )
            }
            CommonEventField::Sender => {
                matches!(self, Self::MessageLike | Self::State | Self::ToDevice)
                    && var != EventVariation::Initial
            }
        }
    }
}

impl From<CommonEventKind> for EventEnumKind {
    fn from(value: CommonEventKind) -> Self {
        match value {
            CommonEventKind::GlobalAccountData => Self::GlobalAccountData,
            CommonEventKind::RoomAccountData => Self::RoomAccountData,
            CommonEventKind::EphemeralRoom => Self::EphemeralRoom,
            CommonEventKind::MessageLike => Self::MessageLike,
            CommonEventKind::State => Self::State,
            CommonEventKind::ToDevice => Self::ToDevice,
        }
    }
}

impl fmt::Display for EventEnumKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(common_kind) = self.common_kind() {
            fmt::Display::fmt(&common_kind, f)
        } else {
            // This is the Timeline kind
            write!(f, "TimelineEvent")
        }
    }
}

/// An entry for an event type in the `event_enum!` macro.
#[derive(Clone)]
struct EventEnumEntry {
    /// The attributes on the event type.
    attrs: Vec<syn::Attribute>,

    /// The types of the event.
    types: EventTypes,

    /// The path to the module containing the event.
    ev_path: syn::Path,

    /// The name of the variant.
    ident: syn::Ident,

    /// Whether this event represents both global and room account data.
    both_account_data: bool,
}

impl EventEnumEntry {
    /// Whether this entry has a type fragment.
    fn has_type_fragment(&self) -> bool {
        self.types.ev_type.is_prefix()
    }

    /// Get or generate the path of the event type for this entry.
    fn to_event_path(&self, kind: EventEnumKind, var: EventVariation) -> syn::Path {
        let type_prefix = match kind {
            EventEnumKind::ToDevice => "ToDevice",
            // Special case event types that represent both account data kinds.
            EventEnumKind::GlobalAccountData if self.both_account_data => "Global",
            EventEnumKind::RoomAccountData if self.both_account_data => "Room",
            // Special case encrypted state event for MSC4362.
            EventEnumKind::State
                if self
                    .types
                    .stable_type()
                    .is_some_and(|ev_type| ev_type.as_str() == "m.room.encrypted") =>
            {
                "State"
            }
            _ => "",
        };

        let event_name = format_ident!("{var}{type_prefix}{}Event", self.ident);

        let mut path = self.ev_path.clone();
        path.segments.push(event_name.into());

        path
    }

    /// Get or generate the path of the event content type for this entry.
    fn to_event_content_path(&self, kind: EventEnumKind) -> syn::Path {
        let type_prefix = match kind {
            EventEnumKind::ToDevice => "ToDevice",
            // Special case encrypted state event for MSC4362.
            EventEnumKind::State
                if self
                    .types
                    .stable_type()
                    .is_some_and(|ev_type| ev_type.as_str() == "m.room.encrypted") =>
            {
                "State"
            }
            _ => "",
        };

        let content_name = format_ident!("{type_prefix}{}EventContent", self.ident);

        let mut path = self.ev_path.clone();
        path.segments.push(content_name.into());

        path
    }

    /// Generate the docs for this entry.
    fn docs(&self) -> TokenStream {
        let main_type = self.types.main_type();

        let mut doc = quote! {
            #[doc = #main_type]
        };

        if self.types.ev_type != *main_type {
            let unstable_name =
                format!("This variant uses the unstable type `{}`.", self.types.ev_type);

            doc.extend(quote! {
                #[doc = ""]
                #[doc = #unstable_name]
            });
        }

        let aliases = &self.types.aliases;
        match aliases.len() {
            0 => {}
            1 => {
                let alias = format!(
                    "This variant can also be deserialized from the `{}` type.",
                    aliases[0]
                );
                doc.extend(quote! {
                    #[doc = ""]
                    #[doc = #alias]
                });
            }
            _ => {
                let aliases = format!(
                    "This variant can also be deserialized from the following types: {}.",
                    aliases.iter().map(|alias| format!("`{alias}`")).collect::<Vec<_>>().join(", ")
                );
                doc.extend(quote! {
                    #[doc = ""]
                    #[doc = #aliases]
                });
            }
        }

        doc
    }
}
