//! Implementations of the MessageEventContent and StateEventContent derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, LitStr, Token,
};

mod kw {
    // This `content` field is kept when the event is redacted.
    syn::custom_keyword!(skip_redaction);
    // Do not emit any redacted event code.
    syn::custom_keyword!(custom_redacted);
}

/// Parses attributes for `*EventContent` derives.
///
/// `#[ruma_event(type = "m.room.alias")]`
#[derive(Eq, PartialEq)]
enum EventMeta {
    /// Variant holds the "m.whatever" event type.
    Type(LitStr),

    /// Fields marked with `#[ruma_event(skip_redaction)]` are kept when the event is
    /// redacted.
    SkipRedacted,

    /// This attribute signals that the events redacted form is manually implemented and should
    /// not be generated.
    CustomRedacted,
}

impl EventMeta {
    fn get_event_type(&self) -> Option<&LitStr> {
        if let Self::Type(lit) = self {
            Some(lit)
        } else {
            None
        }
    }
}

impl Parse for EventMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![type]>().is_ok() {
            input.parse::<Token![=]>()?;
            Ok(EventMeta::Type(input.parse::<LitStr>()?))
        } else if input.parse::<kw::skip_redaction>().is_ok() {
            Ok(EventMeta::SkipRedacted)
        } else if input.parse::<kw::custom_redacted>().is_ok() {
            Ok(EventMeta::CustomRedacted)
        } else {
            Err(syn::Error::new(input.span(), "not a recognized `ruma_event` attribute"))
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

    let event_type = content_attr.iter().find_map(|a| a.get_event_type()).ok_or_else(|| {
        let msg = "no event type attribute found, \
            add `#[ruma_event(type = \"any.room.event\")]` \
            below the event content derive";

        syn::Error::new(Span::call_site(), msg)
    })?;

    let redacted = if needs_redacted(&input) {
        let doc = format!("The payload for a redacted `{}`", ident);
        let redacted_ident = quote::format_ident!("Redacted{}", ident);
        let kept_redacted_fields = if let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) = &input.data
        {
            let mut fields = named
                .iter()
                .filter(|f| {
                    f.attrs.iter().find_map(|a| a.parse_args::<EventMeta>().ok())
                        == Some(EventMeta::SkipRedacted)
                })
                .cloned()
                .collect::<Vec<_>>();
            // don't re-emit our `ruma_event` attributes
            for f in &mut fields {
                f.attrs.retain(|a| !a.path.is_ident("ruma_event"));
            }
            fields
        } else {
            vec![]
        };
        let redaction_struct_fields = kept_redacted_fields.iter().flat_map(|f| &f.ident);

        // redacted_fields allows one to declare an empty redacted event without braces,
        // otherwise `RedactedWhateverEventContent {}` is needed.
        // The redacted_return is used in `EventContent::redacted` which only returns
        // zero sized types (unit structs).
        let (redacted_fields, redacted_return) = if kept_redacted_fields.is_empty() {
            (quote! { ; }, quote! { Ok(#redacted_ident {}) })
        } else {
            (
                quote! {
                    { #( #kept_redacted_fields, )* }
                },
                quote! {
                    Err(::serde::de::Error::custom(
                        format!("this redacted event has fields that cannot be constructed")
                    ))
                },
            )
        };

        let has_fields = if kept_redacted_fields.is_empty() {
            quote! { false }
        } else {
            quote! { true }
        };

        quote! {
            // this is the non redacted event content's impl
            impl #ident {
                /// Transforms the full event content into a redacted content according to spec.
                pub fn redact(self) -> #redacted_ident {
                    #redacted_ident { #( #redaction_struct_fields: self.#redaction_struct_fields, )* }
                }
            }

            #[doc = #doc]
            #[derive(Clone, Debug, ::serde::Deserialize, ::serde::Serialize)]
            pub struct #redacted_ident #redacted_fields

            impl ::ruma_events::EventContent for #redacted_ident {
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

                    Ok(::serde_json::from_str(content.get())?)
                }
            }

            impl ::ruma_events::RedactedEventContent for #redacted_ident {
                fn has_serialize_fields(&self) -> bool {
                    #has_fields
                }

                fn has_deserialize_fields() -> bool {
                    #has_fields
                }

                fn redacted(ev_type: &str) -> Result<Self, ::serde_json::Error> {
                    if ev_type != #event_type {
                        return Err(::serde::de::Error::custom(
                            format!("expected event type `{}`, found `{}`", #event_type, ev_type)
                        ));
                    }

                    #redacted_return
                }
            }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
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
            impl ::ruma_events::RedactedBasicEventContent for #ident { }
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
            impl ::ruma_events::RedactedEphemeralRoomEventContent for #ident { }
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
            impl ::ruma_events::RedactedRoomEventContent for #ident { }
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
            impl ::ruma_events::RedactedMessageEventContent for #ident { }
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
            impl ::ruma_events::RedactedStateEventContent for #ident { }
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
        .find(|a| a == &EventMeta::CustomRedacted)
        .is_none()
}
