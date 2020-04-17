//! Details of generating code for the `ruma_event` procedural macro.
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Field, Ident, LitStr, Token,
};

use crate::parse::{Content, EventKind, RumaEventInput};

/// The result of processing the `ruma_event` macro, ready for output back to source code.
pub struct RumaEvent {
    /// Outer attributes on the field, such as a docstring.
    attrs: Vec<Attribute>,

    /// Information for generating the type used for the event's `content` field.
    content: Content,

    /// The name of the type of the event's `content` field.
    content_name: Ident,

    /// The variant of `ruma_events::EventType` for this event, determined by the `event_type`
    /// field.
    event_type: LitStr,

    /// Struct fields of the event.
    fields: Vec<Field>,

    /// The kind of event.
    kind: EventKind,

    /// The name of the event.
    name: Ident,
}

impl From<RumaEventInput> for RumaEvent {
    fn from(input: RumaEventInput) -> Self {
        let kind = input.kind;
        let name = input.name;
        let content_name = format_ident!("{}Content", name, span = Span::call_site());
        let event_type = input.event_type;

        let mut fields = match kind {
            EventKind::Event => {
                populate_event_fields(content_name.clone(), input.fields.unwrap_or_else(Vec::new))
            }
            EventKind::RoomEvent => populate_room_event_fields(
                content_name.clone(),
                input.fields.unwrap_or_else(Vec::new),
            ),
            EventKind::StateEvent => {
                populate_state_fields(content_name.clone(), input.fields.unwrap_or_else(Vec::new))
            }
        };

        fields.sort_unstable_by_key(|field| field.ident.clone().unwrap());

        Self {
            attrs: input.attrs,
            content: input.content,
            content_name,
            event_type,
            fields,
            kind,
            name,
        }
    }
}

impl ToTokens for RumaEvent {
    // TODO: Maybe break this off into functions so it's not so large. Then remove the clippy
    // allowance.
    #[allow(clippy::cognitive_complexity)]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attrs;
        let content_name = &self.content_name;
        let event_fields = &self.fields;

        let event_type_variant = {
            let event_type = to_camel_case(self.event_type.value());
            let variant = Ident::new(&event_type, event_type.span());

            quote! {
                ::ruma_events::EventType::#variant
            }
        };

        let name = &self.name;
        let content_docstring = format!("The payload for `{}`.", name);

        let content = match &self.content {
            Content::Struct(fields) => {
                quote! {
                    #[doc = #content_docstring]
                    #[derive(Clone, Debug, PartialEq, serde::Serialize)]
                    pub struct #content_name {
                        #(#fields),*
                    }
                }
            }
            Content::Typedef(typedef) => {
                let content_attrs = &typedef.attrs;
                let path = &typedef.path;

                quote! {
                    #(#content_attrs)*
                    pub type #content_name = #path;
                }
            }
        };

        let raw_content = match &self.content {
            Content::Struct(fields) => {
                quote! {
                    #[doc = #content_docstring]
                    #[derive(Clone, Debug, PartialEq, serde::Deserialize)]
                    pub struct #content_name {
                        #(#fields),*
                    }
                }
            }
            Content::Typedef(_) => TokenStream::new(),
        };

        let impl_room_event = match self.kind {
            EventKind::RoomEvent | EventKind::StateEvent => {
                quote! {
                    impl ::ruma_events::RoomEvent for #name {
                        /// The unique identifier for the event.
                        fn event_id(&self) -> &ruma_identifiers::EventId {
                            &self.event_id
                        }

                        /// Time on originating homeserver when this event was sent.
                        fn origin_server_ts(&self) -> std::time::SystemTime {
                            self.origin_server_ts
                        }

                        /// The unique identifier for the room associated with this event.
                        ///
                        /// This can be `None` if the event came from a context where there is
                        /// no ambiguity which room it belongs to, like a `/sync` response for example.
                        fn room_id(&self) -> Option<&ruma_identifiers::RoomId> {
                            self.room_id.as_ref()
                        }

                        /// The unique identifier for the user who sent this event.
                        fn sender(&self) -> &ruma_identifiers::UserId {
                            &self.sender
                        }

                        /// Additional key-value pairs not signed by the homeserver.
                        fn unsigned(&self) -> &serde_json::Map<String, serde_json::Value> {
                            &self.unsigned
                        }
                    }
                }
            }
            _ => TokenStream::new(),
        };

        let impl_state_event = if self.kind == EventKind::StateEvent {
            quote! {
                impl ::ruma_events::StateEvent for #name {
                    /// The previous content for this state key, if any.
                    fn prev_content(&self) -> Option<&Self::Content> {
                        self.prev_content.as_ref()
                    }

                    /// A key that determines which piece of room state the event represents.
                    fn state_key(&self) -> &str {
                        &self.state_key
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        let impl_event_result_compatible_for_content =
            if let Content::Struct(content_fields) = &self.content {
                let mut content_field_values: Vec<TokenStream> =
                    Vec::with_capacity(content_fields.len());

                for content_field in content_fields {
                    let content_field_ident = content_field.ident.clone().unwrap();
                    let span = content_field.span();

                    let token_stream = quote_spanned! {span=>
                        #content_field_ident: raw.#content_field_ident,
                    };

                    content_field_values.push(token_stream);
                }

                quote! {
                    impl ::ruma_events::FromRaw for #content_name {
                        type Raw = raw::#content_name;

                        fn from_raw(
                            raw: raw::#content_name
                        ) -> Self {
                            Self {
                                #(#content_field_values)*
                            }
                        }
                    }
                }
            } else {
                TokenStream::new()
            };

        let event_type_name = self.event_type.value();
        let output = quote!(
            #(#attrs)*
            #[derive(Clone, PartialEq, Debug, serde::Serialize, ruma_events_macros::FromRaw)]
            #[serde(rename = #event_type_name, tag = "type")]
            pub struct #name {
                #(#event_fields),*
            }

            #content

            #impl_event_result_compatible_for_content

            impl ::ruma_events::Event for #name {
                /// The type of this event's `content` field.
                type Content = #content_name;

                /// The event's content.
                fn content(&self) -> &Self::Content {
                    &self.content
                }

                /// The type of the event.
                fn event_type(&self) -> ::ruma_events::EventType {
                    #event_type_variant
                }
            }

            #impl_room_event

            #impl_state_event

            /// "Raw" versions of the event and its content which implement `serde::Deserialize`.
            pub(crate) mod raw {
                use super::*;

                #(#attrs)*
                #[derive(Clone, Debug, PartialEq, serde::Deserialize)]
                pub struct #name {
                    #(#event_fields),*
                }

                #raw_content
            }
        );

        output.to_tokens(tokens);
    }
}

/// Fills in the event's struct definition with fields common to all basic events.
fn populate_event_fields(content_name: Ident, mut fields: Vec<Field>) -> Vec<Field> {
    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The event's content.
        pub content: #content_name,
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// Fills in the event's struct definition with fields common to all room events.
fn populate_room_event_fields(content_name: Ident, fields: Vec<Field>) -> Vec<Field> {
    let mut fields = populate_event_fields(content_name, fields);

    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The unique identifier for the event.
        pub event_id: ruma_identifiers::EventId,

        /// Time on originating homeserver when this event was sent.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: std::time::SystemTime,

        /// The unique identifier for the room associated with this event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_id: Option<ruma_identifiers::RoomId>,

        /// The unique identifier for the user who sent this event.
        pub sender: ruma_identifiers::UserId,

        /// Additional key-value pairs not signed by the homeserver.
        #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
        pub unsigned: serde_json::Map<String, serde_json::Value>,
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// Fills in the event's struct definition with fields common to all state events.
fn populate_state_fields(content_name: Ident, fields: Vec<Field>) -> Vec<Field> {
    let mut fields = populate_room_event_fields(content_name.clone(), fields);

    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The previous content for this state key, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_content: Option<#content_name>,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// Splits the given `event_type` string on `.` and `_` removing the `m.` then
/// camel casing to give the `EventType` variant.
fn to_camel_case(name: String) -> String {
    assert_eq!(&name[..2], "m.");
    name[2..]
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect()
}

/// A wrapper around `syn::Field` that makes it possible to parse `Punctuated<Field, Token![,]>`
/// from a `TokenStream`.
///
/// See https://github.com/dtolnay/syn/issues/651 for more context.
struct ParsableNamedField {
    /// The wrapped `Field`.
    pub field: Field,
}

impl Parse for ParsableNamedField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let field = Field::parse_named(input)?;

        Ok(Self { field })
    }
}
