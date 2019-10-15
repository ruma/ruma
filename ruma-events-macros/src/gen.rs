//! Details of generating code for the `ruma_event` procedural macro.

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Field, Ident, Path, Token, Type,
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

    /// Whether or not the event type is `EventType::Custom`.
    is_custom: bool,

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
        let event_type = input.event_type;
        let is_custom = is_custom_event_type(&event_type);

        let mut fields = match kind {
            EventKind::Event => populate_event_fields(
                is_custom,
                content_name.clone(),
                input.fields.unwrap_or_else(Vec::new),
            ),
            EventKind::RoomEvent => populate_room_event_fields(
                is_custom,
                content_name.clone(),
                input.fields.unwrap_or_else(Vec::new),
            ),
            EventKind::StateEvent => populate_state_fields(
                is_custom,
                content_name.clone(),
                input.fields.unwrap_or_else(Vec::new),
            ),
        };

        fields.sort_unstable_by_key(|field| field.ident.clone().unwrap());

        Self {
            attrs: input.attrs,
            content: input.content,
            content_name,
            event_type,
            fields,
            is_custom,
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

        let event_type = if self.is_custom {
            quote! {
                crate::EventType::Custom(self.event_type.clone())
            }
        } else {
            let event_type = &self.event_type;

            quote! {
                #event_type
            }
        };

        let name = &self.name;
        let name_str = format!("{}", name);
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

        // Custom events will already have an event_type field. All other events need to account
        // for this field being manually inserted in `Serialize` impls.
        let mut base_field_count: usize = if self.is_custom { 0 } else { 1 };

        // Keep track of all the optional fields, because we'll need to check at runtime if they
        // are `Some` in order to increase the number of fields we tell serde to serialize.
        let mut optional_field_idents = Vec::with_capacity(event_fields.len());

        let mut try_from_field_values: Vec<TokenStream> = Vec::with_capacity(event_fields.len());
        let mut serialize_field_calls: Vec<TokenStream> = Vec::with_capacity(event_fields.len());

        for field in event_fields {
            let ident = field.ident.clone().unwrap();

            let ident_str = if ident == "event_type" {
                "type".to_string()
            } else {
                format!("{}", ident)
            };

            let span = field.span();

            let try_from_field_value = if ident == "content" {
                match &self.content {
                    Content::Struct(_) => {
                        quote_spanned! {span=>
                            content: crate::FromRaw::from_raw(raw.content),
                        }
                    }
                    Content::Typedef(_) => {
                        quote_spanned! {span=>
                            content: raw.content,
                        }
                    }
                }
            } else if ident == "prev_content" {
                match &self.content {
                    Content::Struct(_) => {
                        quote_spanned! {span=>
                            prev_content: raw.prev_content.map(crate::FromRaw::from_raw),
                        }
                    }
                    Content::Typedef(_) => {
                        quote_spanned! {span=>
                            prev_content: raw.prev_content,
                        }
                    }
                }
            } else {
                quote_spanned! {span=>
                    #ident: raw.#ident,
                }
            };

            try_from_field_values.push(try_from_field_value);

            // Does the same thing as #[serde(skip_serializing_if = "Option::is_none")]
            let serialize_field_call = if is_option(&field.ty) {
                optional_field_idents.push(ident.clone());

                quote_spanned! {span=>
                    if self.#ident.is_some() {
                        state.serialize_field(#ident_str, &self.#ident)?;
                    }
                }
            } else {
                base_field_count += 1;

                quote_spanned! {span=>
                    state.serialize_field(#ident_str, &self.#ident)?;
                }
            };

            serialize_field_calls.push(serialize_field_call);
        }

        let (manually_serialize_type_field, import_event_in_serialize_impl) = if self.is_custom {
            (TokenStream::new(), TokenStream::new())
        } else {
            let manually_serialize_type_field = quote! {
                state.serialize_field("type", &self.event_type())?;
            };

            let import_event_in_serialize_impl = quote! {
                use crate::Event as _;
            };

            (
                manually_serialize_type_field,
                import_event_in_serialize_impl,
            )
        };

        let increment_struct_len_statements: Vec<TokenStream> = optional_field_idents
            .iter()
            .map(|ident| {
                let span = ident.span();

                quote_spanned! {span=>
                    if self.#ident.is_some() {
                        len += 1;
                    }
                }
            })
            .collect();

        let set_up_struct_serializer = quote! {
            let mut len = #base_field_count;

            #(#increment_struct_len_statements)*

            let mut state = serializer.serialize_struct(#name_str, len)?;
        };

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
                    impl crate::FromRaw for #content_name {
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

        let output = quote!(
            #(#attrs)*
            #[derive(Clone, PartialEq, Debug)]
            pub struct #name {
                #(#event_fields),*
            }

            #content

            impl crate::FromRaw for #name {
                type Raw = raw::#name;

                fn from_raw(raw: raw::#name) -> Self {
                    Self {
                        #(#try_from_field_values)*
                    }
                }
            }

            #impl_event_result_compatible_for_content

            use serde::ser::SerializeStruct as _;

            impl serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer
                {
                    #import_event_in_serialize_impl

                    #set_up_struct_serializer

                    #(#serialize_field_calls)*
                    #manually_serialize_type_field

                    state.end()
                }
            }

            impl crate::Event for #name {
                /// The type of this event's `content` field.
                type Content = #content_name;

                /// The event's content.
                fn content(&self) -> &Self::Content {
                    &self.content
                }

                /// The type of the event.
                fn event_type(&self) -> crate::EventType {
                    #event_type
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
fn populate_event_fields(
    is_custom: bool,
    content_name: Ident,
    mut fields: Vec<Field>,
) -> Vec<Field> {
    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = if is_custom {
        parse_quote! {
            /// The event's content.
            pub content: #content_name,

            /// The custom type of the event.
            pub event_type: String,
        }
    } else {
        parse_quote! {
            /// The event's content.
            pub content: #content_name,
        }
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// Fills in the event's struct definition with fields common to all room events.
fn populate_room_event_fields(
    is_custom: bool,
    content_name: Ident,
    fields: Vec<Field>,
) -> Vec<Field> {
    let mut fields = populate_event_fields(is_custom, content_name, fields);

    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The unique identifier for the event.
        pub event_id: ruma_identifiers::EventId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
        /// event was sent.
        pub origin_server_ts: js_int::UInt,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<ruma_identifiers::RoomId>,

        /// The unique identifier for the user who sent this event.
        pub sender: ruma_identifiers::UserId,

        /// Additional key-value pairs not signed by the homeserver.
        pub unsigned: Option<serde_json::Value>,
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// Fills in the event's struct definition with fields common to all state events.
fn populate_state_fields(is_custom: bool, content_name: Ident, fields: Vec<Field>) -> Vec<Field> {
    let mut fields = populate_room_event_fields(is_custom, content_name.clone(), fields);

    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The previous content for this state key, if any.
        pub prev_content: Option<#content_name>,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// Checks if the given `Path` refers to `EventType::Custom`.
fn is_custom_event_type(event_type: &Path) -> bool {
    event_type.segments.last().unwrap().ident == "Custom"
}

/// Checks if a type is an `Option`.
fn is_option(ty: &Type) -> bool {
    if let Type::Path(ref type_path) = ty {
        type_path.path.segments.first().unwrap().ident == "Option"
    } else {
        panic!("struct field had unexpected non-path type");
    }
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
