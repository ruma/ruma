//! Implementation of the `StateEventContent` derive macro

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
    /// Variant holds the "m.room.whatever" event type.
    Type(LitStr),
}

impl Parse for EventMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![type]>()?;
        input.parse::<Token![=]>()?;
        Ok(EventMeta::Type(input.parse::<LitStr>()?))
    }
}

/// Create a `StateEventContent` implementation for a struct
pub fn expand_state_event(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let event_type_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("ruma_event"))
        .ok_or_else(|| {
            let msg = "no event type attribute found, \
            add `#[ruma_events(type = \"any.room.event\")]` \
            below the event content derive";

            syn::Error::new(Span::call_site(), msg)
        })?;

    let event_type = {
        let event_meta = event_type_attr.parse_args::<EventMeta>()?;
        let EventMeta::Type(lit) = event_meta;
        lit
    };

    let event_content_impl = quote! {
        impl ::ruma_events::EventContent for #ident {
            fn event_type(&self) -> &str {
                #event_type
            }

            fn from_parts(
                ev_type: &str,
                content: Box<::serde_json::value::RawValue>
            ) -> Result<Self, ::ruma_events::InvalidEvent> {
                if ev_type != #event_type {
                    return Err(::ruma_events::InvalidEvent {
                        kind: ::ruma_events::error::InvalidEventKind::Deserialization,
                        message: format!("expected `{}` found {}", #event_type, ev_type),
                    });
                }

                let ev_json = ::ruma_events::EventJson::from(content);
                ev_json.deserialize()
            }
        }
    };

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::RoomEventContent for #ident { }

        impl ::ruma_events::StateEventContent for #ident { }
    })
}
