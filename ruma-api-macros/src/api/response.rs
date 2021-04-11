//! Details of the `response` section of the procedural macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Field, Ident};

use super::metadata::Metadata;

mod incoming;
mod outgoing;

/// The result of processing the `response` section of the macro.
pub(crate) struct Response {
    /// The attributes that will be applied to the struct definition.
    pub attributes: Vec<Attribute>,

    /// The fields of the response.
    pub fields: Vec<ResponseField>,
}

impl Response {
    /// Whether or not this response has any data in the HTTP body.
    fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    /// Whether or not this response has any data in HTTP headers.
    fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    /// Gets the newtype body field, if this response has one.
    fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_body_field)
    }

    /// Gets the newtype raw body field, if this response has one.
    fn newtype_raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_raw_body_field)
    }

    pub(super) fn expand(
        &self,
        metadata: &Metadata,
        error_ty: &TokenStream,
        ruma_api: &TokenStream,
    ) -> TokenStream {
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };

        let docs =
            format!("Data in the response from the `{}` API endpoint.", metadata.name.value());
        let struct_attributes = &self.attributes;

        let response_def = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self.fields.iter().map(|response_field| response_field.field());
            quote! { { #(#fields),* } }
        };

        let def = if let Some(body_field) = self.fields.iter().find(|f| f.is_newtype_body()) {
            let field = Field { ident: None, colon_token: None, ..body_field.field().clone() };

            quote! { (#field); }
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter(|f| f.is_body());

            let fields = fields.map(ResponseField::field);

            quote! { { #(#fields),* } }
        } else {
            quote! { {} }
        };

        let response_body_struct = quote! {
            /// Data in the response body.
            #[derive(Debug, #ruma_serde::Outgoing, #serde::Deserialize, #serde::Serialize)]
            struct ResponseBody #def
        };

        let outgoing_response_impl = self.expand_outgoing(ruma_api);
        let incoming_response_impl = self.expand_incoming(error_ty, ruma_api);

        quote! {
            #[doc = #docs]
            #[derive(Debug, Clone, #ruma_serde::Outgoing, #ruma_serde::_FakeDeriveSerde)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #[incoming_derive(!Deserialize)]
            #( #struct_attributes )*
            pub struct Response #response_def

            #response_body_struct

            #outgoing_response_impl
            #incoming_response_impl
        }
    }
}

/// The types of fields that a response can have.
pub(crate) enum ResponseField {
    /// JSON data in the body of the response.
    Body(Field),

    /// Data in an HTTP header.
    Header(Field, Ident),

    /// A specific data type in the body of the response.
    NewtypeBody(Field),

    /// Arbitrary bytes in the body of the response.
    NewtypeRawBody(Field),
}

impl ResponseField {
    /// Gets the inner `Field` value.
    fn field(&self) -> &Field {
        match self {
            ResponseField::Body(field)
            | ResponseField::Header(field, _)
            | ResponseField::NewtypeBody(field)
            | ResponseField::NewtypeRawBody(field) => field,
        }
    }

    /// Whether or not this response field is a body kind.
    pub(super) fn is_body(&self) -> bool {
        self.as_body_field().is_some()
    }

    /// Whether or not this response field is a header kind.
    fn is_header(&self) -> bool {
        matches!(self, ResponseField::Header(..))
    }

    /// Whether or not this response field is a newtype body kind.
    fn is_newtype_body(&self) -> bool {
        self.as_newtype_body_field().is_some()
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

    /// Return the contained field if this response field is a newtype raw body kind.
    fn as_newtype_raw_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::NewtypeRawBody(field) => Some(field),
            _ => None,
        }
    }
}

/// The types of fields that a response can have, without their values.
pub(crate) enum ResponseFieldKind {
    Body,
    Header,
    NewtypeBody,
    NewtypeRawBody,
}
