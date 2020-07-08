//! Implementations of the MessageEventContent and StateEventContent derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, LitStr, Token,
};

mod kw {
    syn::custom_keyword!(skip_redacted);
    syn::custom_keyword!(custom_redacted);
}

/// Parses attributes for `*EventContent` derives.
///
/// `#[ruma_event(type = "m.room.alias")]`
#[derive(Eq, PartialEq)]
enum EventMeta {
    /// Variant holds the "m.whatever" event type.
    Type(LitStr),

    /// Variant signals that this content type keeps redacted fields and is manually implemented.
    SkipRedacted,

    /// Variant signals when an event is a manually implemented redacted event.
    RedactedCustom,
}

impl EventMeta {
    fn is_event_type(&self) -> bool {
        if let Self::Type(_) = self {
            true
        } else {
            false
        }
    }
}

impl Parse for EventMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![type]>().is_ok() {
            input.parse::<Token![=]>()?;
            Ok(EventMeta::Type(input.parse::<LitStr>()?))
        } else if input.parse::<kw::skip_redacted>().is_ok() {
            Ok(EventMeta::SkipRedacted)
        } else if input.parse::<kw::custom_redacted>().is_ok() {
            Ok(EventMeta::RedactedCustom)
        } else {
            Err(syn::Error::new(input.span(), "not a `ruma_event` attribute"))
        }
    }
}

/// Create an `EventContent` implementation for a struct.
pub fn expand_event_content(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let content_attr = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ruma_event"))
        .map(|attr| attr.parse_args::<EventMeta>())
        .collect::<syn::Result<Vec<_>>>()?;

    let event_type = {
        let event_str = content_attr.iter().find(|a| a.is_event_type()).ok_or_else(|| {
            let msg = "no event type attribute found, \
            add `#[ruma_event(type = \"any.room.event\")]` \
            below the event content derive";

            syn::Error::new(Span::call_site(), msg)
        })?;

        if let EventMeta::Type(lit) = event_str {
            lit
        } else {
            unreachable!("variant was checked to be of type EventMeta::Type")
        }
    };

    let (redact_method, redacted) = if needs_redacted(input) {
        let doc = format!("The payload for a redacted {}", ident);
        let redacted_ident = quote::format_ident!("Redacted{}", ident);

        (
            quote! {
                impl #ident {
                    /// Transforms the full event content into a redacted content according to spec.
                    pub fn redact(self) -> #redacted_ident {
                        #redacted_ident
                    }
                }
            },
            quote! {
                #[doc = #doc]
                #[derive(Clone, Debug, Default, ::serde::Deserialize, ::serde::Serialize)]
                pub struct #redacted_ident;

                impl ::ruma_events::EventContent for #redacted_ident {
                    fn event_type(&self) -> &str {
                        #event_type
                    }

                    fn from_parts(
                        ev_type: &str,
                        content: Box<::serde_json::value::RawValue>
                    ) -> Result<Self, ::serde_json::Error> {
                        // TODO error or just Ok(#redacted_ident) ??
                        Err(::serde::de::Error::custom("redacted event content cannot be generated from a JSON value"))
                    }

                    fn redacted(ev_type: &str) -> Result<Self, ::serde_json::Error> {
                        if ev_type != #event_type {
                            return Err(::serde::de::Error::custom(
                                format!("expected event type `{}`, found `{}`", #event_type, ev_type)
                            ));
                        }

                        Ok(#redacted_ident)
                    }
                }
            },
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    Ok(quote! {
        #redact_method

        impl ::ruma_events::EventContent for #ident {
            fn event_type(&self) -> &str {
                #event_type
            }

            fn from_parts(
                ev_type: &str,
                content: Box<::serde_json::value::RawValue>
            ) -> Result<Self, ::serde_json::Error> {
                if ev_type != #event_type {
                    return Err(::serde::de::Error::custom(
                        format!("expected event type `{}`, found `{}`", #event_type, ev_type)
                    ));
                }

                ::serde_json::from_str(content.get())
            }
        }

        #redacted
    })
}

/// Create a `BasicEventContent` implementation for a struct
pub fn expand_basic_event_content(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let redacted_marker_trait = if needs_redacted(input) {
        let ident = quote::format_ident!("Redacted{}", input.ident);
        quote! {
            impl ::ruma_events::BasicEventContent for #ident { }
        }
    } else {
        TokenStream::new()
    };
    let event_content_impl = expand_event_content(input)?;

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::BasicEventContent for #ident { }

        #redacted_marker_trait
    })
}

/// Create a `EphemeralRoomEventContent` implementation for a struct
pub fn expand_ephemeral_room_event_content(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let event_content_impl = expand_event_content(input)?;

    let redacted_marker_trait = if needs_redacted(input) {
        let ident = quote::format_ident!("Redacted{}", input.ident);
        quote! {
            impl ::ruma_events::EphemeralRoomEventContent for #ident { }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::EphemeralRoomEventContent for #ident { }

        #redacted_marker_trait
    })
}

/// Create a `RoomEventContent` implementation for a struct.
pub fn expand_room_event_content(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let event_content_impl = expand_event_content(input)?;

    let redacted_marker_trait = if needs_redacted(input) {
        let ident = quote::format_ident!("Redacted{}", input.ident);
        quote! {
            impl ::ruma_events::RoomEventContent for #ident { }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #event_content_impl

        impl ::ruma_events::RoomEventContent for #ident { }

        #redacted_marker_trait
    })
}

/// Create a `MessageEventContent` implementation for a struct
pub fn expand_message_event_content(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let room_ev_content = expand_room_event_content(input)?;

    let redacted_marker_trait = if needs_redacted(input) {
        let ident = quote::format_ident!("Redacted{}", &ident);
        quote! {
            impl ::ruma_events::MessageEventContent for #ident { }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #room_ev_content

        impl ::ruma_events::MessageEventContent for #ident { }

        #redacted_marker_trait
    })
}

/// Create a `StateEventContent` implementation for a struct
pub fn expand_state_event_content(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let room_ev_content = expand_room_event_content(input)?;

    let redacted_marker_trait = if needs_redacted(input) {
        let ident = quote::format_ident!("Redacted{}", input.ident);
        quote! {
            impl ::ruma_events::StateEventContent for #ident { }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #room_ev_content

        impl ::ruma_events::StateEventContent for #ident { }

        #redacted_marker_trait
    })
}

fn needs_redacted(input: &DeriveInput) -> bool {
    input
        .attrs
        .iter()
        .flat_map(|a| a.parse_args::<EventMeta>().ok())
        .find(|a| a == &EventMeta::SkipRedacted || a == &EventMeta::RedactedCustom)
        .is_none()
}
