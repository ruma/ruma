use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr};

use super::event_parse::{EventEnumEntry, EventEnumInput, EventKind};

pub fn expand_event_type_enum(
    input: EventEnumInput,
    ruma_common: TokenStream,
) -> syn::Result<TokenStream> {
    let mut room: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut state: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut message: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut ephemeral: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut room_account: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut global_account: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut to_device: Vec<&Vec<EventEnumEntry>> = vec![];
    for event in &input.enums {
        match event.kind {
            EventKind::GlobalAccountData => global_account.push(&event.events),
            EventKind::RoomAccountData => room_account.push(&event.events),
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

    let mut res = TokenStream::new();

    res.extend(generate_enum("EventType", &all, &ruma_common)?);
    res.extend(generate_enum("RoomEventType", &room, &ruma_common)?);
    res.extend(generate_enum("StateEventType", &state, &ruma_common)?);
    res.extend(generate_enum("MessageLikeEventType", &message, &ruma_common)?);
    res.extend(generate_enum("EphemeralRoomEventType", &ephemeral, &ruma_common)?);
    res.extend(generate_enum("RoomAccountDataEventType", &room_account, &ruma_common)?);
    res.extend(generate_enum("GlobalAccountDataEventType", &global_account, &ruma_common)?);
    res.extend(generate_enum("ToDeviceEventType", &to_device, &ruma_common)?);

    Ok(res)
}

fn generate_enum(
    ident: &str,
    input: &[&Vec<EventEnumEntry>],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let str_doc = format!("Creates a string slice from this `{}`.", ident);
    let byte_doc = format!("Creates a byte slice from this `{}`.", ident);
    let enum_doc = format!("The type of `{}` this is.", ident.strip_suffix("Type").unwrap());

    let ident = Ident::new(ident, Span::call_site());

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

    let event_types = deduped.iter().map(|e| &e.ev_type);
    let variants =
        deduped.iter().map(|e| Ok(e.to_variant()?.decl())).collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #[doc = #enum_doc]
        ///
        /// This type can hold an arbitrary string. To check for events that are not available as a
        /// documented variant here, use its string representation, obtained through `.as_str()`.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, #ruma_common::serde::StringEnum)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #event_types]
                #[ruma_enum(rename = #event_types)]
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
    })
}
