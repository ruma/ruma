//! Implementations of the MessageEventContent and StateEventContent derive macro.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Ident, LitStr, Token,
};

use crate::event_parse::EventKind;

mod kw {
    // This `content` field is kept when the event is redacted.
    syn::custom_keyword!(skip_redaction);
    // Do not emit any redacted event code.
    syn::custom_keyword!(custom_redacted);
    // The kind of event content this is.
    syn::custom_keyword!(kind);
}

/// Parses attributes for `*EventContent` derives.
///
/// `#[ruma_event(type = "m.room.alias")]`
enum EventMeta {
    /// Variant holds the "m.whatever" event type.
    Type(LitStr),

    Kind(EventKind),

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

    fn get_event_kind(&self) -> Option<&EventKind> {
        if let Self::Kind(lit) = self {
            Some(lit)
        } else {
            None
        }
    }
}

impl Parse for EventMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![type]) {
            let _: Token![type] = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(EventMeta::Type)
        } else if lookahead.peek(kw::kind) {
            let _: kw::kind = input.parse()?;
            let _: Token![=] = input.parse()?;
            EventKind::parse(input).map(EventMeta::Kind)
        } else if lookahead.peek(kw::skip_redaction) {
            let _: kw::skip_redaction = input.parse()?;
            Ok(EventMeta::SkipRedacted)
        } else if lookahead.peek(kw::custom_redacted) {
            let _: kw::custom_redacted = input.parse()?;
            Ok(EventMeta::CustomRedacted)
        } else {
            Err(lookahead.error())
        }
    }
}

struct MetaAttrs(Vec<EventMeta>);

impl MetaAttrs {
    fn is_custom(&self) -> bool {
        self.0.iter().any(|a| matches!(a, &EventMeta::CustomRedacted))
    }

    fn get_event_type(&self) -> Option<&LitStr> {
        self.0.iter().find_map(|a| a.get_event_type())
    }

    fn get_event_kinds(&self) -> Vec<&EventKind> {
        self.0.iter().filter_map(|a| a.get_event_kind()).collect()
    }
}

impl Parse for MetaAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = syn::punctuated::Punctuated::<EventMeta, Token![,]>::parse_terminated(input)?;
        Ok(Self(attrs.into_iter().collect()))
    }
}

/// Create an `EventContent` implementation for a struct.
pub fn expand_event_content(
    input: &DeriveInput,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let ruma_identifiers = quote! { #ruma_events::exports::ruma_identifiers };
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    let ident = &input.ident;

    let content_attr = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ruma_event"))
        .map(|attr| attr.parse_args::<MetaAttrs>())
        .collect::<syn::Result<Vec<_>>>()?;

    let event_type = content_attr.iter().find_map(|a| a.get_event_type()).ok_or_else(|| {
        let msg = "no event type attribute found, \
            add `#[ruma_event(type = \"any.room.event\", kind = Kind)]` \
            below the event content derive";

        syn::Error::new(Span::call_site(), msg)
    })?;

    let content_derives =
        content_attr.iter().flat_map(|args| args.get_event_kinds()).collect::<Vec<_>>();

    // We only generate redacted content structs for state and message events
    let redacted = if needs_redacted(&content_attr, &content_derives) {
        let doc = format!("The payload for a redacted `{}`", ident);
        let redacted_ident = format_ident!("Redacted{}", ident);
        let kept_redacted_fields = if let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) = &input.data
        {
            // this is to validate the `#[ruma_event(skip_redaction)]` attribute
            named
                .iter()
                .flat_map(|f| &f.attrs)
                .filter(|a| a.path.is_ident("ruma_event"))
                .find_map(|a| {
                    if let Err(e) = a.parse_args::<EventMeta>() {
                        Some(Err(e))
                    } else {
                        None
                    }
                })
                .unwrap_or(Ok(()))?;

            let mut fields: Vec<_> = named
                .iter()
                .filter(|f| {
                    matches!(
                        f.attrs.iter().find_map(|a| a.parse_args::<EventMeta>().ok()),
                        Some(EventMeta::SkipRedacted)
                    )
                })
                .cloned()
                .collect();

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
                    Err(#serde::de::Error::custom(
                        format!("this redacted event has fields that cannot be constructed")
                    ))
                },
            )
        };

        let has_deserialize_fields = if kept_redacted_fields.is_empty() {
            quote! { #ruma_events::HasDeserializeFields::False }
        } else {
            quote! { #ruma_events::HasDeserializeFields::True }
        };

        let has_serialize_fields = if kept_redacted_fields.is_empty() {
            quote! { false }
        } else {
            quote! { true }
        };

        let redacted_event_content =
            generate_event_content_impl(&redacted_ident, event_type, ruma_events);

        let redacted_event_content_derive = content_derives
            .iter()
            .map(|kind| match kind {
                EventKind::Message => quote! {
                    #[automatically_derived]
                    impl #ruma_events::RedactedMessageEventContent for #redacted_ident {}
                },
                EventKind::State => quote! {
                    #[automatically_derived]
                    impl #ruma_events::RedactedStateEventContent for #redacted_ident {}
                },
                _ => TokenStream::new(),
            })
            .collect::<TokenStream>();

        quote! {
            // this is the non redacted event content's impl
            #[automatically_derived]
            impl #ruma_events::RedactContent for #ident {
                type Redacted = #redacted_ident;

                fn redact(self, version: &#ruma_identifiers::RoomVersionId) -> #redacted_ident {
                    #redacted_ident {
                        #( #redaction_struct_fields: self.#redaction_struct_fields, )*
                    }
                }
            }

            #[doc = #doc]
            #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
            pub struct #redacted_ident #redacted_fields

            #redacted_event_content

            #[automatically_derived]
            impl #ruma_events::RedactedEventContent for #redacted_ident {
                fn empty(ev_type: &str) -> #serde_json::Result<Self> {
                    if ev_type != #event_type {
                        return Err(#serde::de::Error::custom(
                            format!("expected event type `{}`, found `{}`", #event_type, ev_type)
                        ));
                    }

                    #redacted_return
                }

                fn has_serialize_fields(&self) -> bool {
                    #has_serialize_fields
                }

                fn has_deserialize_fields() -> #ruma_events::HasDeserializeFields {
                    #has_deserialize_fields
                }
            }

            #redacted_event_content_derive
        }
    } else {
        TokenStream::new()
    };

    let event_content_derive =
        generate_event_content_derives(&content_derives, ident, ruma_events)?;

    let event_content = generate_event_content_impl(ident, event_type, ruma_events);

    Ok(quote! {
        #event_content
        #event_content_derive
        #redacted
    })
}

fn generate_event_content_derives(
    content_attr: &[&EventKind],
    ident: &Ident,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let msg = "valid event kinds are GlobalAccountData, RoomAccountData, \
        EphemeralRoom, Message, State, ToDevice";
    let marker_traits: Vec<_> = content_attr
        .iter()
        .map(|kind| match kind {
            EventKind::GlobalAccountData => Ok(quote! { GlobalAccountDataEventContent }),
            EventKind::RoomAccountData => Ok(quote! { RoomAccountDataEventContent }),
            EventKind::Ephemeral => Ok(quote! { EphemeralRoomEventContent }),
            EventKind::Message => Ok(quote! { MessageEventContent }),
            EventKind::State => Ok(quote! { StateEventContent }),
            EventKind::ToDevice => Ok(quote! { ToDeviceEventContent }),
            EventKind::Redaction => Err(syn::Error::new_spanned(ident, msg)),
            EventKind::Presence => Err(syn::Error::new_spanned(ident, msg)),
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        #(
            #[automatically_derived]
            impl #ruma_events::#marker_traits for #ident {}
        )*
    })
}

fn generate_event_content_impl(
    ident: &Ident,
    event_type: &LitStr,
    ruma_events: &TokenStream,
) -> TokenStream {
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    quote! {
        #[automatically_derived]
        impl #ruma_events::EventContent for #ident {
            fn event_type(&self) -> &str {
                #event_type
            }

            fn from_parts(
                ev_type: &str,
                content: &#serde_json::value::RawValue,
            ) -> #serde_json::Result<Self> {
                if ev_type != #event_type {
                    return Err(#serde::de::Error::custom(
                        format!("expected event type `{}`, found `{}`", #event_type, ev_type)
                    ));
                }

                #serde_json::from_str(content.get())
            }
        }
    }
}

fn needs_redacted(input: &[MetaAttrs], content_derives: &[&EventKind]) -> bool {
    // `is_custom` means that the content struct does not need a generated
    // redacted struct also. If no `custom_redacted` attrs are found the content
    // needs a redacted struct generated.
    !input.iter().any(|a| a.is_custom())
        && content_derives.iter().any(|e| e.is_message() || e.is_state())
}
