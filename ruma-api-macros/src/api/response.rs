//! Details of the `response` section of the procedural macro.

use std::{convert::TryFrom, mem};

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Field, Ident};

use crate::api::{
    attribute::{Meta, MetaNameValue},
    strip_serde_attrs, RawResponse,
};

/// The result of processing the `response` section of the macro.
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

    /// Whether or not this response has any data in the HTTP body.
    pub fn has_body(&self) -> bool {
        self.fields.iter().any(|field| !field.is_header())
    }

    /// Produces code for a response struct initializer.
    pub fn init_fields(&self) -> TokenStream {
        let fields = self.fields.iter().map(|response_field| match response_field {
            ResponseField::Body(field) => {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();

                quote_spanned! {span=>
                    #field_name: response_body.#field_name
                }
            }
            ResponseField::Header(field, header_name) => {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();

                quote_spanned! {span=>
                    #field_name: headers.remove(ruma_api::exports::http::header::#header_name)
                        .expect("response missing expected header")
                        .to_str()
                        .expect("failed to convert HeaderValue to str")
                        .to_owned()
                }
            }
            ResponseField::NewtypeBody(field) => {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
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

    /// Produces code to add necessary HTTP headers to an `http::Response`.
    pub fn apply_header_fields(&self) -> TokenStream {
        let header_calls = self.fields.iter().filter_map(|response_field| {
            if let ResponseField::Header(ref field, ref header_name) = *response_field {
                let field_name = field
                    .ident
                    .as_ref()
                    .expect("expected field to have an identifier");
                let span = field.span();

                Some(quote_spanned! {span=>
                    .header(ruma_api::exports::http::header::#header_name, response.#field_name)
                })
            } else {
                None
            }
        });

        quote! {
            #(#header_calls)*
        }
    }

    /// Produces code to initialize the struct that will be used to create the response body.
    pub fn to_body(&self) -> TokenStream {
        if let Some(field) = self.newtype_body_field() {
            let field_name = field
                .ident
                .as_ref()
                .expect("expected field to have an identifier");
            let span = field.span();
            quote_spanned!(span=> response.#field_name)
        } else {
            let fields = self.fields.iter().filter_map(|response_field| {
                if let ResponseField::Body(ref field) = *response_field {
                    let field_name = field
                        .ident
                        .as_ref()
                        .expect("expected field to have an identifier");
                    let span = field.span();

                    Some(quote_spanned! {span=>
                        #field_name: response.#field_name
                    })
                } else {
                    None
                }
            });

            quote! {
                ResponseBody {
                    #(#fields),*
                }
            }
        }
    }

    /// Gets the newtype body field, if this response has one.
    pub fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_body_field)
    }
}

impl TryFrom<RawResponse> for Response {
    type Error = syn::Error;

    fn try_from(raw: RawResponse) -> syn::Result<Self> {
        let mut newtype_body_field = None;

        let fields = raw
            .fields
            .into_iter()
            .map(|mut field| {
                let mut field_kind = None;
                let mut header = None;

                for attr in mem::replace(&mut field.attrs, Vec::new()) {
                    let meta = match Meta::from_attribute(&attr)? {
                        Some(m) => m,
                        None => {
                            field.attrs.push(attr);
                            continue;
                        }
                    };

                    if field_kind.is_some() {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "There can only be one field kind attribute",
                        ));
                    }

                    field_kind = Some(match meta {
                        Meta::Word(ident) => {
                            if ident != "body" {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "Invalid #[ruma_api] argument with value, expected `body`",
                                ));
                            }

                            if let Some(f) = &newtype_body_field {
                                let mut error = syn::Error::new_spanned(
                                    field,
                                    "There can only be one newtype body field",
                                );
                                error.combine(syn::Error::new_spanned(
                                    f,
                                    "Previous newtype body field",
                                ));
                                return Err(error);
                            }

                            newtype_body_field = Some(field.clone());
                            ResponseFieldKind::NewtypeBody
                        }
                        Meta::NameValue(MetaNameValue { name, value }) => {
                            if name != "header" {
                                return Err(syn::Error::new_spanned(
                                    name,
                                    "Invalid #[ruma_api] argument with value, expected `header`",
                                ));
                            }

                            header = Some(value);
                            ResponseFieldKind::Header
                        }
                    });
                }

                Ok(match field_kind.unwrap_or(ResponseFieldKind::Body) {
                    ResponseFieldKind::Body => ResponseField::Body(field),
                    ResponseFieldKind::Header => {
                        ResponseField::Header(field, header.expect("missing header name"))
                    }
                    ResponseFieldKind::NewtypeBody => ResponseField::NewtypeBody(field),
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        if newtype_body_field.is_some() && fields.iter().any(|f| f.is_body()) {
            return Err(syn::Error::new_spanned(
                // TODO: raw,
                raw.response_kw,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        Ok(Self { fields })
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
            let fields =
                self.fields.iter().map(|response_field| strip_serde_attrs(response_field.field()));

            quote! {
                {
                    #(#fields),*
                }
            }
        };

        let response_body_struct = if let Some(field) = self.newtype_body_field() {
            let ty = &field.ty;
            let span = field.span();

            quote_spanned! {span=>
                /// Data in the response body.
                #[derive(Debug, ruma_api::exports::serde::Deserialize, ruma_api::exports::serde::Serialize)]
                struct ResponseBody(#ty);
            }
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter_map(ResponseField::as_body_field);

            quote! {
                /// Data in the response body.
                #[derive(Debug, ruma_api::exports::serde::Deserialize, ruma_api::exports::serde::Serialize)]
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
        match self {
            ResponseField::Body(field)
            | ResponseField::Header(field, _)
            | ResponseField::NewtypeBody(field) => field,
        }
    }

    /// Whether or not this response field is a body kind.
    fn is_body(&self) -> bool {
        self.as_body_field().is_some()
    }

    /// Whether or not this response field is a header kind.
    fn is_header(&self) -> bool {
        match self {
            ResponseField::Header(..) => true,
            _ => false,
        }
    }

    /// Return the contained field if this response field is a body kind.
    fn as_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::Body(field) => Some(field),
            _ => None,
        }
    }

    /// Return the contained field if this response field is a newtype body kind.
    fn as_newtype_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::NewtypeBody(field) => Some(field),
            _ => None,
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
