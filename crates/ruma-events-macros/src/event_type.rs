use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::event_parse::{EventEnumEntry, EventEnumInputs, EventKind};

pub fn expand_event_type_enum(input: &EventEnumInputs) -> syn::Result<TokenStream> {
    let ruma_serde = quote! { ::ruma_serde };

    let mut rooms: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut state: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut messages: Vec<&Vec<EventEnumEntry>> = vec![];
    for event in &input.enums {
        match event.name {
            EventKind::GlobalAccountData => {}
            EventKind::RoomAccountData => {}
            EventKind::Ephemeral => {}
            EventKind::Message => {
                messages.push(&event.events);
                rooms.push(&event.events);
            }
            EventKind::State => {
                state.push(&event.events);
                rooms.push(&event.events)
            }
            EventKind::ToDevice => {}
            EventKind::Redaction => {}
            EventKind::Presence => {}
        }
    }

    let all = input.enums.iter().map(|e| &e.events).collect::<Vec<_>>();
    let (all_event_types, all_rename_attr) = generate_variants(&all)?;
    let all = generate_enum(
        quote::format_ident!("EventType"),
        all_rename_attr,
        all_event_types,
        &ruma_serde,
    );

    let (room_event_types, room_rename_attr) = generate_variants(&rooms)?;
    let room = generate_enum(
        quote::format_ident!("RoomEventType"),
        room_rename_attr,
        room_event_types,
        &ruma_serde,
    );

    let (state_event_types, state_rename_attr) = generate_variants(&state)?;
    let state = generate_enum(
        quote::format_ident!("StateEventType"),
        state_rename_attr,
        state_event_types,
        &ruma_serde,
    );

    let (message_event_types, message_rename_attr) = generate_variants(&messages)?;
    let message = generate_enum(
        quote::format_ident!("MessageEventType"),
        message_rename_attr,
        message_event_types,
        &ruma_serde,
    );

    Ok(quote! {
        #all

        #room

        #state

        #message
    })
}

fn generate_enum<'a>(
    ident: Ident,
    attrs: impl Iterator<Item = &'a LitStr> + 'a,
    variants: impl Iterator<Item = TokenStream> + 'a,
    ruma_serde: &TokenStream,
) -> TokenStream {
    let str_doc = format!("Creates a str slice from this `{}`.", ident);
    let byte_doc = format!("Creates a byte slice from this `{}`.", ident);
    let enum_doc = format!("The type of `{}` this is.", ident);
    quote! {
        #[doc = #enum_doc]
        ///
        /// This type can hold an arbitrary string. To check for events that are not available as a
        /// documented variant here, use its string representation, obtained through `.as_str()`.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, #ruma_serde::StringEnum)]
        pub enum #ident {
           #(
                #[doc = #attrs]
                #[ruma_enum(rename = #attrs)]
                #variants,
            )*
            #[doc(hidden)]
            _Custom(String),
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
) -> syn::Result<(impl Iterator<Item = TokenStream> + 'a, impl Iterator<Item = &'a LitStr> + 'a)> {
    let rename_attr = input.iter().flat_map(|e| *e).map(|e| &e.ev_type);

    let event_types =
        input.iter().flat_map(|e| *e).map(|e| e.to_variant()).collect::<syn::Result<Vec<_>>>()?;

    Ok((event_types.into_iter().map(|e| e.decl()), rename_attr))
}
