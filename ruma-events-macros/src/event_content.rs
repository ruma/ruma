//! Implementations of the MessageEventContent and StateEventContent derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, LitStr, Token,
};

/// Parses attributes for `*EventContent` derives.
///
/// `#[ruma_event(type = "m.room.alias")]`
enum EventMeta {
    /// Variant holds the "m.whatever" event type.
    Type(LitStr),
}

impl Parse for EventMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![type]>()?;
        input.parse::<Token![=]>()?;
        Ok(EventMeta::Type(input.parse::<LitStr>()?))
    }
}

/// Create an `EventContent` implementation for a struct.
pub fn expand_event_content(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let event_type_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("ruma_event"))
        .ok_or_else(|| {
            let msg = "no event type attribute found, \
            add `#[ruma_event(type = \"any.room.event\")]` \
            below the event content derive";

            syn::Error::new(Span::call_site(), msg)
        })?;

    let event_type = {
        let event_meta = event_type_attr.parse_args::<EventMeta>()?;
        let EventMeta::Type(lit) = event_meta;
        lit
    };

    Ok(quote! {
        impl ::ruma_events::EventContent for #ident {
            fn event_type(&self) -> &str {
                #event_type
            }

            fn from_parts(
                ev_type: &str,
                content: Box<::serde_json::value::RawValue>
            ) -> Result<Self, String> {
                if ev_type != #event_type {
                    return Err(format!("expected `{}` found {}", #event_type, ev_type));
                }

                ::serde_json::from_str(content.get()).map_err(|e| e.to_string())
            }
        }
    })
}

/// Create a `BasicEventContent` implementation for a struct
pub fn expand_basic_event_content(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let event_content_impl = expand_event_content(input)?;

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::BasicEventContent for #ident { }
    })
}

/// Create a `EphemeralRoomEventContent` implementation for a struct
pub fn expand_ephemeral_room_event_content(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let event_content_impl = expand_event_content(input)?;

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::EphemeralRoomEventContent for #ident { }
    })
}

/// Create a `RoomEventContent` implementation for a struct.
pub fn expand_room_event_content(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let event_content_impl = expand_event_content(input)?;

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::RoomEventContent for #ident { }
    })
}

/// Create a `MessageEventContent` implementation for a struct
pub fn expand_message_event_content(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let room_ev_content = expand_room_event_content(input)?;

    Ok(quote! {
        #room_ev_content

        impl ::ruma_events::MessageEventContent for #ident { }
    })
}

/// Create a `StateEventContent` implementation for a struct
pub fn expand_state_event_content(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let room_ev_content = expand_room_event_content(input)?;

    Ok(quote! {
        #room_ev_content

        impl ::ruma_events::StateEventContent for #ident { }
    })
}
