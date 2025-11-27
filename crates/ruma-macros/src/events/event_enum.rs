//! Implementation of the `event_enum!` macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod event_kind_enum;
mod event_type;
mod parse;
mod util;

use self::{
    event_kind_enum::EventEnum, event_type::EventTypeEnum, util::expand_json_castable_impl,
};
use super::enums::{EventKind, EventTypes, EventVariation};
use crate::util::RumaEvents;

/// Generates enums to represent the various Matrix event types.
pub fn expand_event_enum(input: EventEnumInput) -> syn::Result<TokenStream> {
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
                    kind: EventKind::Timeline,
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
        for variation in data.kind.event_enum_variations() {
            let ident = data.kind.to_event_enum_ident(*variation)?;

            tokens.extend(expand_json_castable_impl(&ident, data.kind, *variation, &ruma_events)?);
        }

        event_enums_data.push(data);
    }

    tokens.extend(event_enums_data.iter().map(|data| {
        EventTypeEnum::new(data, &ruma_events)
            .expand()
            .unwrap_or_else(syn::Error::into_compile_error)
    }));

    Ok(tokens)
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

    /// The event kind.
    kind: EventKind,

    /// The event types for this kind.
    events: Vec<EventEnumEntry>,
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
    fn to_event_path(&self, kind: EventKind, var: EventVariation) -> syn::Path {
        let type_prefix = match kind {
            EventKind::ToDevice => "ToDevice",
            // Special case event types that represent both account data kinds.
            EventKind::GlobalAccountData if self.both_account_data => "Global",
            EventKind::RoomAccountData if self.both_account_data => "Room",
            // Special case encrypted state event for MSC4362.
            EventKind::State
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
    fn to_event_content_path(&self, kind: EventKind) -> syn::Path {
        let type_prefix = match kind {
            EventKind::ToDevice => "ToDevice",
            // Special case encrypted state event for MSC4362.
            EventKind::State
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
