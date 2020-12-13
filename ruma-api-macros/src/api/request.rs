//! Details of the `request` section of the procedural macro.

use std::{collections::BTreeSet, mem};

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Field, Ident, Lifetime, Token,
};

use crate::{
    api::attribute::{Meta, MetaNameValue},
    util,
};

mod kw {
    syn::custom_keyword!(request);
}

#[derive(Debug, Default)]
pub struct RequestLifetimes {
    body: BTreeSet<Lifetime>,
    path: BTreeSet<Lifetime>,
    query: BTreeSet<Lifetime>,
    header: BTreeSet<Lifetime>,
}

/// The result of processing the `request` section of the macro.
pub struct Request {
    /// The attributes that will be applied to the struct definition.
    attributes: Vec<Attribute>,

    /// The fields of the request.
    fields: Vec<RequestField>,

    /// The collected lifetime identifiers from the declared fields.
    lifetimes: RequestLifetimes,

    // Guarantee `ruma_api` is available and named something we can refer to.
    ruma_api_import: TokenStream,
}

impl Request {
    /// Produces code to add necessary HTTP headers to an `http::Request`.
    pub fn append_header_kvs(&self) -> TokenStream {
        let ruma_api = &self.ruma_api_import;
        let http = quote! { #ruma_api::exports::http };

        self.header_fields()
            .map(|request_field| {
                let (field, header_name) = match request_field {
                    RequestField::Header(field, header_name) => (field, header_name),
                    _ => unreachable!("expected request field to be header variant"),
                };

                let field_name = &field.ident;

                match &field.ty {
                    syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                        if segments.last().unwrap().ident == "Option" =>
                    {
                        quote! {
                            if let Some(header_val) = self.#field_name.as_ref() {
                                req_builder = req_builder.header(
                                    #http::header::#header_name,
                                    #http::header::HeaderValue::from_str(header_val)?,
                                );
                            }
                        }
                    }
                    _ => quote! {
                        req_builder = req_builder.header(
                            #http::header::#header_name,
                            #http::header::HeaderValue::from_str(self.#field_name.as_ref())?,
                        );
                    },
                }
            })
            .collect()
    }

    /// Produces code to extract fields from the HTTP headers in an `http::Request`.
    pub fn parse_headers_from_request(&self) -> TokenStream {
        let ruma_api = &self.ruma_api_import;
        let http = quote! { #ruma_api::exports::http };
        let serde = quote! { #ruma_api::exports::serde };
        let serde_json = quote! { #ruma_api::exports::serde_json };

        let fields = self.header_fields().map(|request_field| {
            let (field, header_name) = match request_field {
                RequestField::Header(field, header_name) => (field, header_name),
                _ => panic!("expected request field to be header variant"),
            };

            let field_name = &field.ident;
            let header_name_string = header_name.to_string();

            let (some_case, none_case) = match &field.ty {
                syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                    if segments.last().unwrap().ident == "Option" =>
                {
                    (quote! { Some(header.to_owned()) }, quote! { None })
                }
                _ => (
                    quote! { header.to_owned() },
                    quote! {{
                        use #serde::de::Error as _;

                        // FIXME: Not a missing json field, a missing header!
                        return Err(#ruma_api::error::RequestDeserializationError::new(
                            #serde_json::Error::missing_field(
                                #header_name_string
                            ),
                            request,
                        )
                        .into());
                    }},
                ),
            };

            quote! {
                #field_name: match headers
                    .get(#http::header::#header_name)
                    .and_then(|v| v.to_str().ok()) // FIXME: Should have a distinct error message
                {
                    Some(header) => #some_case,
                    None => #none_case,
                }
            }
        });

        quote! {
            #(#fields,)*
        }
    }

    /// Whether or not this request has any data in the HTTP body.
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    /// Whether or not this request has any data in HTTP headers.
    pub fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    /// Whether or not this request has any data in the URL path.
    pub fn has_path_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_path())
    }

    /// Whether or not this request has any data in the query string.
    pub fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_query())
    }

    /// Produces an iterator over all the body fields.
    pub fn body_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter_map(|field| field.as_body_field())
    }

    /// The number of unique lifetime annotations for `body` fields.
    pub fn body_lifetime_count(&self) -> usize {
        self.lifetimes.body.len()
    }

    /// Whether any `body` field has a lifetime annotation.
    pub fn has_body_lifetimes(&self) -> bool {
        !self.lifetimes.body.is_empty()
    }

    /// Whether any `query` field has a lifetime annotation.
    pub fn has_query_lifetimes(&self) -> bool {
        !self.lifetimes.query.is_empty()
    }

    /// Whether any field has a lifetime.
    pub fn contains_lifetimes(&self) -> bool {
        !(self.lifetimes.body.is_empty()
            && self.lifetimes.path.is_empty()
            && self.lifetimes.query.is_empty()
            && self.lifetimes.header.is_empty())
    }

    /// The combination of every fields unique lifetime annotation.
    pub fn combine_lifetimes(&self) -> TokenStream {
        util::unique_lifetimes_to_tokens(
            self.lifetimes
                .body
                .iter()
                .chain(self.lifetimes.path.iter())
                .chain(self.lifetimes.query.iter())
                .chain(self.lifetimes.header.iter())
                .collect::<BTreeSet<_>>()
                .into_iter(),
        )
    }

    /// The lifetimes on fields with the `query` attribute.
    pub fn query_lifetimes(&self) -> TokenStream {
        util::unique_lifetimes_to_tokens(self.lifetimes.query.iter())
    }

    /// The lifetimes on fields with the `body` attribute.
    pub fn body_lifetimes(&self) -> TokenStream {
        util::unique_lifetimes_to_tokens(self.lifetimes.body.iter())
    }

    // /// The lifetimes on fields with the `header` attribute.
    // pub fn header_lifetimes(&self) -> TokenStream {
    //     util::generics_to_tokens(self.lifetimes.header.iter())
    // }

    /// Produces an iterator over all the header fields.
    pub fn header_fields(&self) -> impl Iterator<Item = &RequestField> {
        self.fields.iter().filter(|field| field.is_header())
    }

    /// Gets the number of path fields.
    pub fn path_field_count(&self) -> usize {
        self.fields.iter().filter(|field| field.is_path()).count()
    }

    /// Returns the body field.
    pub fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_newtype_body_field)
    }

    /// Returns the body field.
    pub fn newtype_raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_newtype_raw_body_field)
    }

    /// Returns the query map field.
    pub fn query_map_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_query_map_field)
    }

    /// Produces code for a struct initializer for body fields on a variable named `request`.
    pub fn request_body_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Body, quote!(self))
    }

    /// Produces code for a struct initializer for query string fields on a variable named
    /// `request`.
    pub fn request_query_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Query, quote!(self))
    }

    /// Produces code for a struct initializer for body fields on a variable named `request_body`.
    pub fn request_init_body_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Body, quote!(request_body))
    }

    /// Produces code for a struct initializer for query string fields on a variable named
    /// `request_query`.
    pub fn request_init_query_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Query, quote!(request_query))
    }

    /// Produces code for a struct initializer for the given field kind to be accessed through the
    /// given variable name.
    fn struct_init_fields(
        &self,
        request_field_kind: RequestFieldKind,
        src: TokenStream,
    ) -> TokenStream {
        let process_field = |f: &RequestField| {
            f.field_of_kind(request_field_kind).map(|field| {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                quote_spanned! {span=>
                    #( #cfg_attrs )*
                    #field_name: #src.#field_name
                }
            })
        };

        let mut fields = vec![];
        let mut new_type_body = None;
        for field in &self.fields {
            if let RequestField::NewtypeRawBody(_) = field {
                new_type_body = process_field(field);
            } else {
                fields.extend(process_field(field));
            }
        }

        // Move field that consumes `request` to the end of the init list.
        fields.extend(new_type_body);

        quote! { #(#fields,)* }
    }
}

impl Parse for Request {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let request_kw = input.parse::<kw::request>()?;
        input.parse::<Token![:]>()?;
        let fields;
        braced!(fields in input);

        let fields = fields.parse_terminated::<Field, Token![,]>(Field::parse_named)?;

        let mut newtype_body_field = None;
        let mut query_map_field = None;
        let mut lifetimes = RequestLifetimes::default();

        let fields = fields
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
                        Meta::Word(ident) => match &ident.to_string()[..] {
                            attr @ "body" | attr @ "raw_body" => util::req_res_meta_word(
                                attr,
                                &field,
                                &mut newtype_body_field,
                                RequestFieldKind::NewtypeBody,
                                RequestFieldKind::NewtypeRawBody,
                            )?,
                            "path" => RequestFieldKind::Path,
                            "query" => RequestFieldKind::Query,
                            "query_map" => {
                                if let Some(f) = &query_map_field {
                                    let mut error = syn::Error::new_spanned(
                                        field,
                                        "There can only be one query map field",
                                    );
                                    error.combine(syn::Error::new_spanned(
                                        f,
                                        "Previous query map field",
                                    ));
                                    return Err(error);
                                }

                                query_map_field = Some(field.clone());
                                RequestFieldKind::QueryMap
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "Invalid #[ruma_api] argument, expected one of \
                                     `body`, `path`, `query`, `query_map`",
                                ));
                            }
                        },
                        Meta::NameValue(MetaNameValue { name, value }) => util::req_res_name_value(
                            name,
                            value,
                            &mut header,
                            RequestFieldKind::Header,
                        )?,
                    });
                }

                match field_kind.unwrap_or(RequestFieldKind::Body) {
                    RequestFieldKind::Header => {
                        util::collect_lifetime_ident(&mut lifetimes.header, &field.ty)
                    }
                    RequestFieldKind::Body => {
                        util::collect_lifetime_ident(&mut lifetimes.body, &field.ty)
                    }
                    RequestFieldKind::NewtypeBody => {
                        util::collect_lifetime_ident(&mut lifetimes.body, &field.ty)
                    }
                    RequestFieldKind::NewtypeRawBody => {
                        util::collect_lifetime_ident(&mut lifetimes.body, &field.ty)
                    }
                    RequestFieldKind::Path => {
                        util::collect_lifetime_ident(&mut lifetimes.path, &field.ty)
                    }
                    RequestFieldKind::Query => {
                        util::collect_lifetime_ident(&mut lifetimes.query, &field.ty)
                    }
                    RequestFieldKind::QueryMap => {
                        util::collect_lifetime_ident(&mut lifetimes.query, &field.ty)
                    }
                }

                Ok(RequestField::new(field_kind.unwrap_or(RequestFieldKind::Body), field, header))
            })
            .collect::<syn::Result<Vec<_>>>()?;

        if newtype_body_field.is_some() && fields.iter().any(|f| f.is_body()) {
            // TODO: highlight conflicting fields,
            return Err(syn::Error::new_spanned(
                request_kw,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        if query_map_field.is_some() && fields.iter().any(|f| f.is_query()) {
            return Err(syn::Error::new_spanned(
                // TODO: raw,
                request_kw,
                "Can't have both a query map field and regular query fields",
            ));
        }

        // TODO when/if `&[(&str, &str)]` is supported remove this
        if query_map_field.is_some() && !lifetimes.query.is_empty() {
            return Err(syn::Error::new_spanned(
                request_kw,
                "Lifetimes are not allowed for query_map fields",
            ));
        }

        Ok(Self { attributes, fields, lifetimes, ruma_api_import: util::import_ruma_api() })
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ruma_api = &self.ruma_api_import;
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };

        let struct_attributes = &self.attributes;

        let request_def = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self.fields.iter().map(|request_field| request_field.field());
            quote! { { #(#fields),* } }
        };

        let request_generics = self.combine_lifetimes();

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

        let request = quote! {
            #[derive(Debug, Clone, #ruma_serde::Outgoing, #ruma_serde::_FakeDeriveSerde)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #[incoming_derive(!Deserialize)]
            #( #struct_attributes )*
            pub struct Request #request_generics #request_def

            #request_body_struct

            #request_query_struct
        };

        request.to_tokens(tokens);
    }
}

/// The types of fields that a request can have.
pub enum RequestField {
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
    fn new(kind: RequestFieldKind, field: Field, header: Option<Ident>) -> Self {
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

    /// Gets the kind of the request field.
    fn kind(&self) -> RequestFieldKind {
        match self {
            RequestField::Body(..) => RequestFieldKind::Body,
            RequestField::Header(..) => RequestFieldKind::Header,
            RequestField::NewtypeBody(..) => RequestFieldKind::NewtypeBody,
            RequestField::NewtypeRawBody(..) => RequestFieldKind::NewtypeRawBody,
            RequestField::Path(..) => RequestFieldKind::Path,
            RequestField::Query(..) => RequestFieldKind::Query,
            RequestField::QueryMap(..) => RequestFieldKind::QueryMap,
        }
    }

    /// Whether or not this request field is a body kind.
    fn is_body(&self) -> bool {
        self.kind() == RequestFieldKind::Body
    }

    /// Whether or not this request field is a header kind.
    fn is_header(&self) -> bool {
        self.kind() == RequestFieldKind::Header
    }

    /// Whether or not this request field is a newtype body kind.
    fn is_newtype_body(&self) -> bool {
        self.kind() == RequestFieldKind::NewtypeBody
    }

    /// Whether or not this request field is a path kind.
    fn is_path(&self) -> bool {
        self.kind() == RequestFieldKind::Path
    }

    /// Whether or not this request field is a query string kind.
    fn is_query(&self) -> bool {
        self.kind() == RequestFieldKind::Query
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
        if self.kind() == kind {
            Some(self.field())
        } else {
            None
        }
    }
}

/// The types of fields that a request can have, without their values.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestFieldKind {
    /// See the similarly named variant of `RequestField`.
    Body,

    /// See the similarly named variant of `RequestField`.
    Header,

    /// See the similarly named variant of `RequestField`.
    NewtypeBody,

    /// See the similarly named variant of `RequestField`.
    NewtypeRawBody,

    /// See the similarly named variant of `RequestField`.
    Path,

    /// See the similarly named variant of `RequestField`.
    Query,

    /// See the similarly named variant of `RequestField`.
    QueryMap,
}
