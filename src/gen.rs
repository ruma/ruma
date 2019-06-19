//! Details of generating code for the `ruma_event` procedural macro.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    Attribute, Field, Ident, Token,
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

    /// Struct fields of the event.
    fields: Vec<Field>,

    /// The kind of event.
    #[allow(dead_code)]
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
            fields,
            kind,
            name,
        }
    }
}

impl ToTokens for RumaEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attrs;
        let content_name = &self.content_name;

        let event_fields = &self.fields;

        let name = &self.name;

        let content_docstring = format!("The payload for `{}`.", name);

        let content = match &self.content {
            Content::Struct(fields) => {
                quote! {
                    #[doc = #content_docstring]
                    #[derive(Clone, Debug)]
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

        let output = quote!(
            #(#attrs)*
            #[derive(Clone, Debug)]
            pub struct #name {
                #(#event_fields),*
            }

            #content

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
