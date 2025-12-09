//! Parsing helpers specific to the `event_enum!` macro.

use std::collections::BTreeMap;

use proc_macro2::Span;
use syn::{
    meta::ParseNestedMeta,
    parse::{Parse, ParseStream},
};

use super::{EventEnumData, EventEnumEntry, EventEnumInput};
use crate::events::common::{CommonEventKind, EventType, EventTypes};

impl Parse for EventEnumInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut enums_map = BTreeMap::new();

        while !input.is_empty() {
            let attrs = input.call(syn::Attribute::parse_outer)?;

            let _: syn::Token![enum] = input.parse()?;
            let kind: CommonEventKind = input.parse()?;

            let content;
            syn::braced!(content in input);
            let events = content.parse_terminated(EventEnumEntry::parse, syn::Token![,])?;
            let events = events.into_iter().collect();

            if enums_map.insert(kind, EventEnumData { attrs, kind: kind.into(), events }).is_some()
            {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("duplicate definition for kind `{kind:?}`"),
                ));
            }
        }

        // Mark event types which are declared for both account data kinds, because they use a
        // different name for the event struct.
        let mut room_account_data = enums_map.remove(&CommonEventKind::RoomAccountData);
        if let Some((global_account_data, room_account_data)) =
            enums_map.get_mut(&CommonEventKind::GlobalAccountData).zip(room_account_data.as_mut())
        {
            for global_event in global_account_data.events.iter_mut() {
                if let Some(room_event) = room_account_data.events.iter_mut().find(|room_event| {
                    room_event.types.ev_type == global_event.types.ev_type
                        && room_event.ev_path == global_event.ev_path
                        && room_event.ident == global_event.ident
                }) {
                    global_event.both_account_data = true;
                    room_event.both_account_data = true;
                }
            }
        }
        if let Some(room_account_data) = room_account_data {
            enums_map.insert(CommonEventKind::RoomAccountData, room_account_data);
        }

        Ok(EventEnumInput { enums: enums_map.into_values().collect() })
    }
}

impl Parse for EventEnumEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (ruma_enum_attrs, attrs) = input
            .call(syn::Attribute::parse_outer)?
            .into_iter()
            .partition::<Vec<_>, _>(|attr| attr.path().is_ident("ruma_enum"));
        let ev_type: EventType = input.parse()?;
        let _: syn::Token![=>] = input.parse()?;
        let ev_path = input.call(syn::Path::parse_mod_style)?;

        let mut entry_attrs = EventEnumEntryAttrs::default();

        for attr in ruma_enum_attrs {
            attr.parse_nested_meta(|meta| entry_attrs.try_merge(meta, &attr))?;
        }

        let types = EventTypes::try_from_parts(ev_type, entry_attrs.aliases)?;

        // We will need the name of the event type so compute it right now to make sure that we have
        // enough data for it.
        let ident =
            if let Some(ident) = entry_attrs.ident { ident } else { types.as_event_ident()? };

        Ok(Self { attrs, types, ev_path, ident, both_account_data: false })
    }
}

/// Parsed attributes on an event entry in the `event_enum!` macro.
#[derive(Default)]
struct EventEnumEntryAttrs {
    /// The custom name of the variant.
    ident: Option<syn::Ident>,

    /// The alternative event types.
    aliases: Vec<EventType>,
}

impl EventEnumEntryAttrs {
    /// Set the name of the Rust event type.
    ///
    /// Returns an error if the name is already set.
    fn set_ident(&mut self, ident: syn::Ident, attr: &syn::Attribute) -> syn::Result<()> {
        if self.ident.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `ident` attribute",
            ));
        }

        self.ident = Some(ident);
        Ok(())
    }

    /// Try to parse the given meta item and merge it into this `EventEnumEntryAttrs`.
    ///
    /// Returns an error if parsing the meta item fails, or if it sets a field that was already set.
    pub(crate) fn try_merge(
        &mut self,
        meta: ParseNestedMeta<'_>,
        attr: &syn::Attribute,
    ) -> syn::Result<()> {
        if meta.path.is_ident("ident") {
            return self.set_ident(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("alias") {
            self.aliases.push(meta.value()?.parse()?);
            return Ok(());
        }

        Err(meta.error("unsupported `ruma_enum` attribute"))
    }
}
