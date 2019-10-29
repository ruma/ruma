//! Details of the `response` section of the procedural macro.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Field, Ident};

use crate::api::{
    attribute::{Meta, MetaNameValue},
    strip_serde_attrs,
};

/// The result of processing the `request` section of the macro.
pub struct Response {
    /// The fields of the response.
    fields: Vec<ResponseField>,
}

impl Response {
    /// Whether or not this response has any data in the HTTP body.
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    /// Whether or not this response has any fields.
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }

    /// Whether or not this response has any data in HTTP headers.
    pub fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    /// Produces code for a request struct initializer.
    pub fn init_fields(&self) -> TokenStream {
        let fields = self
            .fields
            .iter()
            .map(|response_field| match *response_field {
                ResponseField::Body(ref field) => {
                    let field_name = field
                        .ident
                        .clone()
                        .expect("expected field to have an identifier");
                    let span = field.span();

                    quote_spanned! {span=>
                        #field_name: response_body.#field_name
                    }
                }
                ResponseField::Header(ref field, ref header_name) => {
                    let field_name = field
                        .ident
                        .clone()
                        .expect("expected field to have an identifier");
                    let span = field.span();

                    quote_spanned! {span=>
                        #field_name: headers.remove(ruma_api::exports::http::header::#header_name)
                            .expect("response missing expected header")
                            .to_str()
                            .expect("failed to convert HeaderValue to str")
                            .to_owned()
                    }
                }
                ResponseField::NewtypeBody(ref field) => {
                    let field_name = field
                        .ident
                        .clone()
                        .expect("expected field to have an identifier");
                    let span = field.span();

                    quote_spanned! {span=>
                        #field_name: response_body
                    }
                }
            });

        quote! {
            #(#fields,)*
        }
    }

    /// Gets the newtype body field, if this request has one.
    pub fn newtype_body_field(&self) -> Option<&Field> {
        for response_field in self.fields.iter() {
            match *response_field {
                ResponseField::NewtypeBody(ref field) => {
                    return Some(field);
                }
                _ => continue,
            }
        }

        None
    }
}

impl From<Vec<Field>> for Response {
    fn from(fields: Vec<Field>) -> Self {
        let mut has_newtype_body = false;

        let fields = fields.into_iter().map(|mut field| {
            let mut field_kind = ResponseFieldKind::Body;
            let mut header = None;

            field.attrs = field.attrs.into_iter().filter_map(|attr| {
                let meta = match Meta::from_attribute(attr) {
                    Ok(meta) => meta,
                    Err(attr) => return Some(attr),
                };

                match meta {
                    Meta::Word(ident) => {
                        assert!(
                            ident == "body",
                            "ruma_api! single-word attribute on responses must be: body"
                        );

                        has_newtype_body = true;
                        field_kind = ResponseFieldKind::NewtypeBody;
                    }
                    Meta::NameValue(MetaNameValue { name, value }) => {
                        assert!(
                            name == "header",
                            "ruma_api! name/value pair attribute on requests must be: header"
                        );

                        header = Some(value);
                        field_kind = ResponseFieldKind::Header;
                    }
                }

                None
            }).collect();

            match field_kind {
                ResponseFieldKind::Body => {
                    assert!(
                        !has_newtype_body,
                        "ruma_api! responses cannot have both normal body fields and a newtype body field"
                    );

                    ResponseField::Body(field)
                }
                ResponseFieldKind::Header => ResponseField::Header(field, header.expect("missing header name")),
                ResponseFieldKind::NewtypeBody => ResponseField::NewtypeBody(field),
            }
        }).collect();

        Self { fields }
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let response_struct_header = quote! {
            #[derive(Debug, Clone)]
            pub struct Response
        };

        let response_struct_body = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self.fields.iter().map(|response_field| {
                let field = response_field.field();
                let span = field.span();

                let stripped_field = strip_serde_attrs(field);

                quote_spanned!(span=> #stripped_field)
            });

            quote! {
                {
                    #(#fields),*
                }
            }
        };

        let response_body_struct = if let Some(newtype_body_field) = self.newtype_body_field() {
            let field = newtype_body_field.clone();
            let ty = &field.ty;
            let span = field.span();

            quote_spanned! {span=>
                /// Data in the response body.
                #[derive(Debug, ruma_api::exports::serde::Deserialize)]
                struct ResponseBody(#ty);
            }
        } else if self.has_body_fields() {
            let fields = self
                .fields
                .iter()
                .filter_map(|response_field| match *response_field {
                    ResponseField::Body(ref field) => {
                        let span = field.span();
                        Some(quote_spanned!(span=> #field))
                    }
                    _ => None,
                });

            quote! {
                /// Data in the response body.
                #[derive(Debug, ruma_api::exports::serde::Deserialize)]
                struct ResponseBody {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let response = quote! {
            #response_struct_header
            #response_struct_body
            #response_body_struct
        };

        response.to_tokens(tokens);
    }
}

/// The types of fields that a response can have.
pub enum ResponseField {
    /// JSON data in the body of the response.
    Body(Field),
    /// Data in an HTTP header.
    Header(Field, Ident),
    /// A specific data type in the body of the response.
    NewtypeBody(Field),
}

impl ResponseField {
    /// Gets the inner `Field` value.
    fn field(&self) -> &Field {
        match *self {
            ResponseField::Body(ref field)
            | ResponseField::Header(ref field, _)
            | ResponseField::NewtypeBody(ref field) => field,
        }
    }

    /// Whether or not this response field is a body kind.
    fn is_body(&self) -> bool {
        match *self {
            ResponseField::Body(_) => true,
            _ => false,
        }
    }

    /// Whether or not this response field is a header kind.
    fn is_header(&self) -> bool {
        match *self {
            ResponseField::Header(..) => true,
            _ => false,
        }
    }
}

/// The types of fields that a response can have, without their values.
enum ResponseFieldKind {
    /// See the similarly named variant of `ResponseField`.
    Body,
    /// See the similarly named variant of `ResponseField`.
    Header,
    /// See the similarly named variant of `ResponseField`.
    NewtypeBody,
}
