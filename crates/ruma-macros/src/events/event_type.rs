use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Ident, LitStr};

use super::event_parse::{EventEnumEntry, EventEnumInput, EventKind};

pub fn expand_event_type_enum(
    input: EventEnumInput,
    ruma_common: TokenStream,
) -> syn::Result<TokenStream> {
    let ruma_serde = quote! { #ruma_common::exports::ruma_serde };

    let mut room: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut state: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut message: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut ephemeral: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut to_device: Vec<&Vec<EventEnumEntry>> = vec![];
    for event in &input.enums {
        match event.kind {
            EventKind::Ephemeral => ephemeral.push(&event.events),
            EventKind::MessageLike => {
                message.push(&event.events);
                room.push(&event.events);
            }
            EventKind::State => {
                state.push(&event.events);
                room.push(&event.events);
            }
            EventKind::ToDevice => to_device.push(&event.events),
            EventKind::RoomRedaction
            | EventKind::Presence
            | EventKind::Decrypted
            | EventKind::HierarchySpaceChild => {}
        }
    }
    let presence = vec![EventEnumEntry {
        attrs: vec![],
        ev_type: LitStr::new("m.presence", Span::call_site()),
    }];
    let mut all = input.enums.iter().map(|e| &e.events).collect::<Vec<_>>();
    all.push(&presence);
    let (all_event_types, all_str_ev_types) = generate_variants(&all)?;
    let all =
        generate_enum(format_ident!("EventType"), all_str_ev_types, all_event_types, &ruma_serde);

    let (room_event_types, room_str_ev_types) = generate_variants(&room)?;
    let room = generate_enum(
        format_ident!("RoomEventType"),
        room_str_ev_types,
        room_event_types,
        &ruma_serde,
    );

    let (state_event_types, state_str_ev_types) = generate_variants(&state)?;
    let state = generate_enum(
        format_ident!("StateEventType"),
        state_str_ev_types,
        state_event_types,
        &ruma_serde,
    );

    let (message_event_types, message_str_ev_types) = generate_variants(&message)?;
    let message = generate_enum(
        format_ident!("MessageLikeEventType"),
        message_str_ev_types,
        message_event_types,
        &ruma_serde,
    );

    let (ephemeral_event_types, ephemeral_str_ev_types) = generate_variants(&ephemeral)?;
    let ephemeral = generate_enum(
        format_ident!("EphemeralRoomEventType"),
        ephemeral_str_ev_types,
        ephemeral_event_types,
        &ruma_serde,
    );

    let (to_device_event_types, to_device_str_ev_types) = generate_variants(&to_device)?;
    let to_device = generate_enum(
        format_ident!("ToDeviceEventType"),
        to_device_str_ev_types,
        to_device_event_types,
        &ruma_serde,
    );

    Ok(quote! {
        #all
        #room
        #state
        #message
        #ephemeral
        #to_device
    })
}

fn generate_enum(
    ident: Ident,
    ev_type_strings: Vec<&LitStr>,
    variants: Vec<TokenStream>,
    ruma_serde: &TokenStream,
) -> TokenStream {
    let str_doc = format!("Creates a string slice from this `{}`.", ident);
    let byte_doc = format!("Creates a byte slice from this `{}`.", ident);
    let enum_doc = format!("The type of `{}` this is.", ident);
    quote! {
        #[doc = #enum_doc]
        ///
        /// This type can hold an arbitrary string. To check for events that are not available as a
        /// documented variant here, use its string representation, obtained through `.as_str()`.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, #ruma_serde::StringEnum)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #ev_type_strings]
                #[ruma_enum(rename = #ev_type_strings)]
                #variants,
            )*
            #[doc(hidden)]
            _Custom(crate::PrivOwnedStr),
        }

        impl #ident {
            #[doc = #str_doc]
            pub fn as_str(&self) -> &str {
                self.as_ref()
            }

            #[doc = #byte_doc]
            pub fn as_bytes(&self) -> &[u8] {
                self.as_str().as_bytes()
            }
        }
    }
}

fn generate_variants<'a>(
    input: &'a [&Vec<EventEnumEntry>],
) -> syn::Result<(Vec<TokenStream>, Vec<&'a LitStr>)> {
    let mut deduped: Vec<&EventEnumEntry> = vec![];
    for item in input.iter().copied().flatten() {
        if let Some(idx) = deduped.iter().position(|e| e.ev_type == item.ev_type) {
            // If there is a variant without config attributes use that
            if deduped[idx].attrs != item.attrs && item.attrs.is_empty() {
                deduped[idx] = item;
            }
        } else {
            deduped.push(item);
        }
    }
    let event_types = deduped.iter().map(|e| &e.ev_type).collect();

    let event_types_variants =
        deduped.iter().map(|e| Ok(e.to_variant()?.decl())).collect::<syn::Result<Vec<_>>>()?;

    Ok((event_types_variants, event_types))
}
