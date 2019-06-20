//! Details of generating code for the `ruma_event` procedural macro.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    Attribute, Field, Ident, Path, Token,
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
    event_type: Path,

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
        let content_name = Ident::new(&format!("{}Content", &name), Span::call_site());

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
            event_type: input.event_type,
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
        let event_type = &self.event_type;
        let name = &self.name;
        let name_str = format!("{}", name);
        let content_docstring = format!("The payload for `{}`.", name);

        let content = match &self.content {
            Content::Struct(fields) => {
                quote! {
                    #[doc = #content_docstring]
                    #[derive(Clone, Debug, serde::Serialize)]
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
                    #[derive(Clone, Debug, serde::Deserialize)]
                    pub struct #content_name {
                        #(#fields),*
                    }
                }
            }
            Content::Typedef(_) => TokenStream::new(),
        };

        let field_count = event_fields.len() + 1; // + 1 because of manually adding `event_type`

        let mut from_str_field_values: Vec<TokenStream> = Vec::with_capacity(event_fields.len());
        let mut serialize_field_calls: Vec<TokenStream> = Vec::with_capacity(event_fields.len());

        for field in event_fields {
            let ident = field.ident.clone().unwrap();
            let ident_str = format!("{}", ident);

            let from_str_field_value = if ident == "content" {
                match &self.content {
                    Content::Struct(content_fields) => {
                        let mut content_field_values: Vec<TokenStream> =
                            Vec::with_capacity(content_fields.len());

                        for content_field in content_fields {
                            let content_field_ident = content_field.ident.clone().unwrap();

                            let token_stream = quote! {
                                #content_field_ident: raw.content.#content_field_ident,
                            };

                            content_field_values.push(token_stream);
                        }

                        quote! {
                            content: #content_name {
                                #(#content_field_values)*
                            },
                        }
                    }
                    Content::Typedef(_) => {
                        quote! {
                            content: raw.content,
                        }
                    }
                }
            } else if ident == "prev_content" {
                match &self.content {
                    Content::Struct(content_fields) => {
                        let mut content_field_values: Vec<TokenStream> =
                            Vec::with_capacity(content_fields.len());

                        for content_field in content_fields {
                            let content_field_ident = content_field.ident.clone().unwrap();

                            let token_stream = quote! {
                                #content_field_ident: prev.#content_field_ident,
                            };

                            content_field_values.push(token_stream);
                        }

                        quote! {
                            prev_content: raw.prev_content.map(|prev| {
                                #content_name {
                                    #(#content_field_values)*
                                }
                            }),
                        }
                    }
                    Content::Typedef(_) => {
                        quote! {
                            content: raw.content,
                        }
                    }
                }
            } else {
                quote! {
                    #ident: raw.#ident,
                }
            };

            from_str_field_values.push(from_str_field_value);

            let serialize_field_call = quote! {
                state.serialize_field(#ident_str, &self.#ident)?;
            };

            serialize_field_calls.push(serialize_field_call);
        }

        let impl_room_event = match self.kind {
            EventKind::RoomEvent | EventKind::StateEvent => {
                quote! {
                    impl crate::RoomEvent for #name {
                        /// The unique identifier for the event.
                        fn event_id(&self) -> &ruma_identifiers::EventId {
                            &self.event_id
                        }

                        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this event was
                        /// sent.
                        fn origin_server_ts(&self) -> js_int::UInt {
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
                        fn unsigned(&self) -> Option<&serde_json::Value> {
                            self.unsigned.as_ref()
                        }
                    }
                }
            }
            _ => TokenStream::new(),
        };

        let impl_state_event = if self.kind == EventKind::StateEvent {
            quote! {
                impl crate::StateEvent for #name {
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

        let output = quote!(
            #(#attrs)*
            #[derive(Clone, Debug)]
            pub struct #name {
                #(#event_fields),*
            }

            #content

            impl #name {
                /// Attempt to create `Self` from parsing a string of JSON data.
                pub fn from_str(json: &str) -> Result<Self, crate::InvalidEvent> {
                    let raw = serde_json::from_str::<raw::#name>(json)?;

                    Ok(Self {
                        #(#from_str_field_values)*
                    })
                }
            }

            use serde::ser::SerializeStruct as _;

            impl serde::Serialize  for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    use crate::Event as _;

                    let mut state = serializer.serialize_struct(#name_str, #field_count)?;

                    #(#serialize_field_calls)*
                    state.serialize_field("type", &self.event_type())?;

                    state.end()
                }
            }

            impl crate::Event for #name {
                /// The type of the event.
                const EVENT_TYPE: crate::EventType = #event_type;

                /// The type of this event's `content` field.
                type Content = #content_name;

                /// The event's content.
                fn content(&self) -> &Self::Content {
                    &self.content
                }
            }

            #impl_room_event

            #impl_state_event

            /// "Raw" versions of the event and its content which implement `serde::Deserialize`.
            mod raw {
                use super::*;

                #(#attrs)*
                #[derive(Clone, Debug, serde::Deserialize)]
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

    let mut additional_fields = Vec::with_capacity(punctuated_fields.len());

    for punctuated_field in punctuated_fields {
        additional_fields.push(punctuated_field.field);
    }

    fields.extend(additional_fields);

    fields
}

/// Fills in the event's struct definition with fields common to all room events.
fn populate_room_event_fields(content_name: Ident, fields: Vec<Field>) -> Vec<Field> {
    let mut fields = populate_event_fields(content_name, fields);

    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The unique identifier for the event.
        pub event_id: ruma_identifiers::EventId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
        /// event was sent.
        pub origin_server_ts: js_int::UInt,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<ruma_identifiers::RoomId>,

        /// Additional key-value pairs not signed by the homeserver.
        pub unsigned: Option<serde_json::Value>,

        /// The unique identifier for the user who sent this event.
        pub sender: ruma_identifiers::UserId,
    };

    let mut additional_fields = Vec::with_capacity(punctuated_fields.len());

    for punctuated_field in punctuated_fields {
        additional_fields.push(punctuated_field.field);
    }

    fields.extend(additional_fields);

    fields
}

/// Fills in the event's struct definition with fields common to all state events.
fn populate_state_fields(content_name: Ident, fields: Vec<Field>) -> Vec<Field> {
    let mut fields = populate_room_event_fields(content_name.clone(), fields);

    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The previous content for this state key, if any.
        pub prev_content: Option<#content_name>,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
    };

    let mut additional_fields = Vec::with_capacity(punctuated_fields.len());

    for punctuated_field in punctuated_fields {
        additional_fields.push(punctuated_field.field);
    }

    fields.extend(additional_fields);

    fields
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
