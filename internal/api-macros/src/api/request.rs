//! Details of the `request` section of the procedural macro.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Field, Ident, Lifetime};

use crate::util;

use super::metadata::Metadata;

mod incoming;
mod outgoing;

#[derive(Debug, Default)]
pub(super) struct RequestLifetimes {
    pub body: BTreeSet<Lifetime>,
    pub path: BTreeSet<Lifetime>,
    pub query: BTreeSet<Lifetime>,
    pub header: BTreeSet<Lifetime>,
}

/// The result of processing the `request` section of the macro.
pub(crate) struct Request {
    /// The attributes that will be applied to the struct definition.
    pub(super) attributes: Vec<Attribute>,

    /// The fields of the request.
    pub(super) fields: Vec<RequestField>,

    /// The collected lifetime identifiers from the declared fields.
    pub(super) lifetimes: RequestLifetimes,
}

impl Request {
    /// Whether or not this request has any data in the HTTP body.
    pub(super) fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    /// Whether or not this request has any data in HTTP headers.
    fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    /// Whether or not this request has any data in the URL path.
    fn has_path_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_path())
    }

    /// Whether or not this request has any data in the query string.
    fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_query())
    }

    /// Produces an iterator over all the body fields.
    pub(super) fn body_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter_map(|field| field.as_body_field())
    }

    /// Whether any `body` field has a lifetime annotation.
    fn has_body_lifetimes(&self) -> bool {
        !self.lifetimes.body.is_empty()
    }

    /// Whether any `query` field has a lifetime annotation.
    fn has_query_lifetimes(&self) -> bool {
        !self.lifetimes.query.is_empty()
    }

    /// Whether any field has a lifetime.
    fn contains_lifetimes(&self) -> bool {
        !(self.lifetimes.body.is_empty()
            && self.lifetimes.path.is_empty()
            && self.lifetimes.query.is_empty()
            && self.lifetimes.header.is_empty())
    }

    /// The combination of every fields unique lifetime annotation.
    fn combine_lifetimes(&self) -> TokenStream {
        util::unique_lifetimes_to_tokens(
            [
                &self.lifetimes.body,
                &self.lifetimes.path,
                &self.lifetimes.query,
                &self.lifetimes.header,
            ]
            .iter()
            .flat_map(|set| set.iter()),
        )
    }

    /// The lifetimes on fields with the `query` attribute.
    fn query_lifetimes(&self) -> TokenStream {
        util::unique_lifetimes_to_tokens(&self.lifetimes.query)
    }

    /// The lifetimes on fields with the `body` attribute.
    fn body_lifetimes(&self) -> TokenStream {
        util::unique_lifetimes_to_tokens(&self.lifetimes.body)
    }

    /// Produces an iterator over all the header fields.
    fn header_fields(&self) -> impl Iterator<Item = &RequestField> {
        self.fields.iter().filter(|field| field.is_header())
    }

    /// Gets the number of path fields.
    fn path_field_count(&self) -> usize {
        self.fields.iter().filter(|field| field.is_path()).count()
    }

    /// Returns the body field.
    pub fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_newtype_body_field)
    }

    /// Returns the body field.
    fn newtype_raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_newtype_raw_body_field)
    }

    /// Returns the query map field.
    fn query_map_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_query_map_field)
    }

    pub(super) fn expand(
        &self,
        metadata: &Metadata,
        error_ty: &TokenStream,
        ruma_api: &TokenStream,
    ) -> TokenStream {
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };

        let docs = format!(
            "Data for a request to the `{}` API endpoint.\n\n{}",
            metadata.name.value(),
            metadata.description.value(),
        );
        let struct_attributes = &self.attributes;

        let request_def = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self.fields.iter().map(|request_field| request_field.field());
            quote! { { #(#fields),* } }
        };

        let request_body_struct =
            if let Some(body_field) = self.fields.iter().find(|f| f.is_newtype_body()) {
                let field = Field { ident: None, colon_token: None, ..body_field.field().clone() };
                // Though we don't track the difference between new type body and body
                // for lifetimes, the outer check and the macro failing if it encounters
                // an illegal combination of field attributes, is enough to guarantee
                // `body_lifetimes` correctness.
                let (derive_deserialize, lifetimes) = if self.has_body_lifetimes() {
                    (TokenStream::new(), self.body_lifetimes())
                } else {
                    (quote!(#serde::Deserialize), TokenStream::new())
                };

                Some((derive_deserialize, quote! { #lifetimes (#field); }))
            } else if self.has_body_fields() {
                let fields = self.fields.iter().filter(|f| f.is_body());
                let (derive_deserialize, lifetimes) = if self.has_body_lifetimes() {
                    (TokenStream::new(), self.body_lifetimes())
                } else {
                    (quote!(#serde::Deserialize), TokenStream::new())
                };
                let fields = fields.map(RequestField::field);

                Some((derive_deserialize, quote! { #lifetimes { #(#fields),* } }))
            } else {
                None
            }
            .map(|(derive_deserialize, def)| {
                quote! {
                    /// Data in the request body.
                    #[derive(
                        Debug,
                        #ruma_serde::Outgoing,
                        #serde::Serialize,
                        #derive_deserialize
                    )]
                    struct RequestBody #def
                }
            });

        let request_query_struct = if let Some(f) = self.query_map_field() {
            let field = Field { ident: None, colon_token: None, ..f.clone() };
            let (derive_deserialize, lifetime) = if self.has_query_lifetimes() {
                (TokenStream::new(), self.query_lifetimes())
            } else {
                (quote!(#serde::Deserialize), TokenStream::new())
            };

            quote! {
                /// Data in the request's query string.
                #[derive(
                    Debug,
                    #ruma_serde::Outgoing,
                    #serde::Serialize,
                    #derive_deserialize
                )]
                struct RequestQuery #lifetime (#field);
            }
        } else if self.has_query_fields() {
            let fields = self.fields.iter().filter_map(RequestField::as_query_field);
            let (derive_deserialize, lifetime) = if self.has_query_lifetimes() {
                (TokenStream::new(), self.query_lifetimes())
            } else {
                (quote!(#serde::Deserialize), TokenStream::new())
            };

            quote! {
                /// Data in the request's query string.
                #[derive(
                    Debug,
                    #ruma_serde::Outgoing,
                    #serde::Serialize,
                    #derive_deserialize
                )]
                struct RequestQuery #lifetime {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let lifetimes = self.combine_lifetimes();
        let outgoing_request_impl = self.expand_outgoing(metadata, error_ty, &lifetimes, ruma_api);
        let incoming_request_impl = self.expand_incoming(metadata, error_ty, ruma_api);

        quote! {
            #[doc = #docs]
            #[derive(Debug, Clone, #ruma_serde::Outgoing, #ruma_serde::_FakeDeriveSerde)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #[incoming_derive(!Deserialize)]
            #( #struct_attributes )*
            pub struct Request #lifetimes #request_def

            #request_body_struct
            #request_query_struct

            #outgoing_request_impl
            #incoming_request_impl
        }
    }
}

/// The types of fields that a request can have.
pub(crate) enum RequestField {
    /// JSON data in the body of the request.
    Body(Field),

    /// Data in an HTTP header.
    Header(Field, Ident),

    /// A specific data type in the body of the request.
    NewtypeBody(Field),

    /// Arbitrary bytes in the body of the request.
    NewtypeRawBody(Field),

    /// Data that appears in the URL path.
    Path(Field),

    /// Data that appears in the query string.
    Query(Field),

    /// Data that appears in the query string as dynamic key-value pairs.
    QueryMap(Field),
}

impl RequestField {
    /// Creates a new `RequestField`.
    pub(super) fn new(kind: RequestFieldKind, field: Field, header: Option<Ident>) -> Self {
        match kind {
            RequestFieldKind::Body => RequestField::Body(field),
            RequestFieldKind::Header => {
                RequestField::Header(field, header.expect("missing header name"))
            }
            RequestFieldKind::NewtypeBody => RequestField::NewtypeBody(field),
            RequestFieldKind::NewtypeRawBody => RequestField::NewtypeRawBody(field),
            RequestFieldKind::Path => RequestField::Path(field),
            RequestFieldKind::Query => RequestField::Query(field),
            RequestFieldKind::QueryMap => RequestField::QueryMap(field),
        }
    }

    /// Whether or not this request field is a body kind.
    pub(super) fn is_body(&self) -> bool {
        matches!(self, RequestField::Body(..))
    }

    /// Whether or not this request field is a header kind.
    fn is_header(&self) -> bool {
        matches!(self, RequestField::Header(..))
    }

    /// Whether or not this request field is a newtype body kind.
    fn is_newtype_body(&self) -> bool {
        matches!(self, RequestField::NewtypeBody(..))
    }

    /// Whether or not this request field is a path kind.
    fn is_path(&self) -> bool {
        matches!(self, RequestField::Path(..))
    }

    /// Whether or not this request field is a query string kind.
    pub(super) fn is_query(&self) -> bool {
        matches!(self, RequestField::Query(..))
    }

    /// Return the contained field if this request field is a body kind.
    fn as_body_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::Body)
    }

    /// Return the contained field if this request field is a body kind.
    fn as_newtype_body_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::NewtypeBody)
    }

    /// Return the contained field if this request field is a raw body kind.
    fn as_newtype_raw_body_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::NewtypeRawBody)
    }

    /// Return the contained field if this request field is a query kind.
    fn as_query_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::Query)
    }

    /// Return the contained field if this request field is a query map kind.
    fn as_query_map_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::QueryMap)
    }

    /// Gets the inner `Field` value.
    fn field(&self) -> &Field {
        match self {
            RequestField::Body(field)
            | RequestField::Header(field, _)
            | RequestField::NewtypeBody(field)
            | RequestField::NewtypeRawBody(field)
            | RequestField::Path(field)
            | RequestField::Query(field)
            | RequestField::QueryMap(field) => field,
        }
    }

    /// Gets the inner `Field` value if it's of the provided kind.
    fn field_of_kind(&self, kind: RequestFieldKind) -> Option<&Field> {
        match (self, kind) {
            (RequestField::Body(field), RequestFieldKind::Body)
            | (RequestField::Header(field, _), RequestFieldKind::Header)
            | (RequestField::NewtypeBody(field), RequestFieldKind::NewtypeBody)
            | (RequestField::NewtypeRawBody(field), RequestFieldKind::NewtypeRawBody)
            | (RequestField::Path(field), RequestFieldKind::Path)
            | (RequestField::Query(field), RequestFieldKind::Query)
            | (RequestField::QueryMap(field), RequestFieldKind::QueryMap) => Some(field),
            _ => None,
        }
    }
}

/// The types of fields that a request can have, without their values.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestFieldKind {
    Body,
    Header,
    NewtypeBody,
    NewtypeRawBody,
    Path,
    Query,
    QueryMap,
}
