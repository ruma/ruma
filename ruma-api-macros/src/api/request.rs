//! Details of the `request` section of the procedural macro.

use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Field, Ident, Lifetime};

use crate::util;

use super::metadata::Metadata;

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

    /// The number of unique lifetime annotations for `body` fields.
    fn body_lifetime_count(&self) -> usize {
        self.lifetimes.body.len()
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

    /// Produces code for a struct initializer for the given field kind to be accessed through the
    /// given variable name.
    fn struct_init_fields(
        &self,
        request_field_kind: RequestFieldKind,
        src: TokenStream,
    ) -> TokenStream {
        let fields =
            self.fields.iter().filter_map(|f| f.field_of_kind(request_field_kind)).map(|field| {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                quote_spanned! {span=>
                    #( #cfg_attrs )*
                    #field_name: #src.#field_name
                }
            });

        quote! { #(#fields,)* }
    }

    fn vars(
        &self,
        request_field_kind: RequestFieldKind,
        src: TokenStream,
    ) -> (TokenStream, TokenStream) {
        let (decls, names): (TokenStream, Vec<_>) = self
            .fields
            .iter()
            .filter_map(|f| f.field_of_kind(request_field_kind))
            .map(|field| {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                let decl = quote_spanned! {span=>
                    #( #cfg_attrs )*
                    let #field_name = #src.#field_name;
                };

                (decl, field_name)
            })
            .unzip();

        let names = quote! { #(#names,)* };

        (decls, names)
    }

    pub(super) fn expand(
        &self,
        metadata: &Metadata,
        error_ty: &TokenStream,
        ruma_api: &TokenStream,
    ) -> TokenStream {
        let bytes = quote! { #ruma_api::exports::bytes };
        let http = quote! { #ruma_api::exports::http };
        let percent_encoding = quote! { #ruma_api::exports::percent_encoding };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };
        let serde_json = quote! { #ruma_api::exports::serde_json };

        let method = &metadata.method;

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

        let incoming_request_type =
            if self.contains_lifetimes() { quote!(IncomingRequest) } else { quote!(Request) };

        let (request_path_string, parse_request_path, path_vars) = if self.has_path_fields() {
            let path_string = metadata.path.value();

            assert!(path_string.starts_with('/'), "path needs to start with '/'");
            assert!(
                path_string.chars().filter(|c| *c == ':').count() == self.path_field_count(),
                "number of declared path parameters needs to match amount of placeholders in path"
            );

            let format_call = {
                let mut format_string = path_string.clone();
                let mut format_args = Vec::new();

                while let Some(start_of_segment) = format_string.find(':') {
                    // ':' should only ever appear at the start of a segment
                    assert_eq!(&format_string[start_of_segment - 1..start_of_segment], "/");

                    let end_of_segment = match format_string[start_of_segment..].find('/') {
                        Some(rel_pos) => start_of_segment + rel_pos,
                        None => format_string.len(),
                    };

                    let path_var = Ident::new(
                        &format_string[start_of_segment + 1..end_of_segment],
                        Span::call_site(),
                    );
                    format_args.push(quote! {
                        #percent_encoding::utf8_percent_encode(
                            &self.#path_var.to_string(),
                            #percent_encoding::NON_ALPHANUMERIC,
                        )
                    });
                    format_string.replace_range(start_of_segment..end_of_segment, "{}");
                }

                quote! {
                    format_args!(#format_string, #(#format_args),*)
                }
            };

            let path_var_decls = path_string[1..]
                .split('/')
                .enumerate()
                .filter(|(_, seg)| seg.starts_with(':'))
                .map(|(i, seg)| {
                    let path_var = Ident::new(&seg[1..], Span::call_site());
                    quote! {
                        let #path_var = {
                            let segment = path_segments[#i].as_bytes();
                            let decoded =
                                #percent_encoding::percent_decode(segment).decode_utf8()?;

                            ::std::convert::TryFrom::try_from(&*decoded)?
                        };
                    }
                });

            let parse_request_path = quote! {
                let path_segments: ::std::vec::Vec<&::std::primitive::str> =
                    request.uri().path()[1..].split('/').collect();

                #(#path_var_decls)*
            };

            let path_vars = path_string[1..]
                .split('/')
                .filter(|seg| seg.starts_with(':'))
                .map(|seg| Ident::new(&seg[1..], Span::call_site()));

            (format_call, parse_request_path, quote! { #(#path_vars,)* })
        } else {
            (quote! { metadata.path.to_owned() }, TokenStream::new(), TokenStream::new())
        };

        let request_query_string = if let Some(field) = self.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have identifier");

            quote!({
                // This function exists so that the compiler will throw an error when the type of
                // the field with the query_map attribute doesn't implement
                // `IntoIterator<Item = (String, String)>`.
                //
                // This is necessary because the `ruma_serde::urlencoded::to_string` call will
                // result in a runtime error when the type cannot be encoded as a list key-value
                // pairs (?key1=value1&key2=value2).
                //
                // By asserting that it implements the iterator trait, we can ensure that it won't
                // fail.
                fn assert_trait_impl<T>(_: &T)
                where
                    T: ::std::iter::IntoIterator<
                        Item = (::std::string::String, ::std::string::String),
                    >,
                {}

                let request_query = RequestQuery(self.#field_name);
                assert_trait_impl(&request_query.0);

                format_args!(
                    "?{}",
                    #ruma_serde::urlencoded::to_string(request_query)?
                )
            })
        } else if self.has_query_fields() {
            let request_query_init_fields =
                self.struct_init_fields(RequestFieldKind::Query, quote!(self));

            quote!({
                let request_query = RequestQuery {
                    #request_query_init_fields
                };

                format_args!(
                    "?{}",
                    #ruma_serde::urlencoded::to_string(request_query)?
                )
            })
        } else {
            quote! { "" }
        };

        let (parse_query, query_vars) = if let Some(field) = self.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = #ruma_serde::urlencoded::from_str(
                    &request.uri().query().unwrap_or(""),
                )?;
            };

            (parse, quote! { #field_name, })
        } else if self.has_query_fields() {
            let (decls, names) = self.vars(RequestFieldKind::Query, quote!(request_query));

            let parse = quote! {
                let request_query: <RequestQuery as #ruma_serde::Outgoing>::Incoming =
                    #ruma_serde::urlencoded::from_str(
                        &request.uri().query().unwrap_or("")
                    )?;

                #decls
            };

            (parse, names)
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let mut header_kvs: TokenStream = self
            .header_fields()
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
                                req_headers.insert(
                                    #http::header::#header_name,
                                    #http::header::HeaderValue::from_str(header_val)?,
                                );
                            }
                        }
                    }
                    _ => quote! {
                        req_headers.insert(
                            #http::header::#header_name,
                            #http::header::HeaderValue::from_str(self.#field_name.as_ref())?,
                        );
                    },
                }
            })
            .collect();

        for auth in &metadata.authentication {
            if auth.value == "AccessToken" {
                let attrs = &auth.attrs;
                header_kvs.extend(quote! {
                    #( #attrs )*
                    req_headers.insert(
                        #http::header::AUTHORIZATION,
                        #http::header::HeaderValue::from_str(
                            &::std::format!(
                                "Bearer {}",
                                access_token.ok_or(
                                    #ruma_api::error::IntoHttpError::NeedsAuthentication
                                )?
                            )
                        )?
                    );
                });
            }
        }

        let (parse_headers, header_vars) = if self.has_header_fields() {
            let (decls, names): (TokenStream, Vec<_>) = self
                .header_fields()
                .map(|request_field| {
                    let (field, header_name) = match request_field {
                        RequestField::Header(field, header_name) => (field, header_name),
                        _ => panic!("expected request field to be header variant"),
                    };

                    let field_name = &field.ident;
                    let header_name_string = header_name.to_string();

                    let (some_case, none_case) = match &field.ty {
                        syn::Type::Path(syn::TypePath {
                            path: syn::Path { segments, .. }, ..
                        }) if segments.last().unwrap().ident == "Option" => {
                            (quote! { Some(str_value.to_owned()) }, quote! { None })
                        }
                        _ => (
                            quote! { str_value.to_owned() },
                            quote! {
                                return Err(
                                    #ruma_api::error::HeaderDeserializationError::MissingHeader(
                                        #header_name_string.into()
                                    ).into(),
                                )
                            },
                        ),
                    };

                    let decl = quote! {
                        let #field_name = match headers.get(#http::header::#header_name) {
                            Some(header_value) => {
                                let str_value = header_value.to_str()?;
                                #some_case
                            }
                            None => #none_case,
                        };
                    };

                    (decl, field_name)
                })
                .unzip();

            let parse = quote! {
                let headers = request.headers();

                #decls
            };

            (parse, quote! { #(#names,)* })
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let extract_body = if self.has_body_fields() || self.newtype_body_field().is_some() {
            let body_lifetimes = if self.has_body_lifetimes() {
                // duplicate the anonymous lifetime as many times as needed
                let lifetimes = std::iter::repeat(quote! { '_ }).take(self.body_lifetime_count());
                quote! { < #( #lifetimes, )* >}
            } else {
                TokenStream::new()
            };

            quote! {
                let request_body: <
                    RequestBody #body_lifetimes
                    as #ruma_serde::Outgoing
                >::Incoming = {
                    let body = request.into_body();
                    if #bytes::Buf::has_remaining(&body) {
                        #serde_json::from_reader(#bytes::Buf::reader(body))?
                    } else {
                        // If the request body is completely empty, pretend it is an empty JSON
                        // object instead. This allows requests with only optional body parameters
                        // to be deserialized in that case.
                        #serde_json::from_str("{}")?
                    }
                };
            }
        } else {
            TokenStream::new()
        };

        let request_body = if let Some(field) = self.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { self.#field_name }
        } else if self.has_body_fields() || self.newtype_body_field().is_some() {
            let request_body_initializers = if let Some(field) = self.newtype_body_field() {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                quote! { (self.#field_name) }
            } else {
                let initializers = self.struct_init_fields(RequestFieldKind::Body, quote!(self));
                quote! { { #initializers } }
            };

            quote! {
                {
                    let request_body = RequestBody #request_body_initializers;
                    #serde_json::to_vec(&request_body)?
                }
            }
        } else {
            quote! { Vec::new() }
        };

        let (parse_body, body_vars) = if let Some(field) = self.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = request_body.0;
            };

            (parse, quote! { #field_name, })
        } else if let Some(field) = self.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = {
                    let mut reader = #bytes::Buf::reader(request.into_body());
                    let mut vec = ::std::vec::Vec::new();
                    ::std::io::Read::read_to_end(&mut reader, &mut vec)
                        .expect("reading from a bytes::Buf never fails");
                    vec
                };
            };

            (parse, quote! { #field_name, })
        } else {
            self.vars(RequestFieldKind::Body, quote!(request_body))
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

        let request_lifetimes = self.combine_lifetimes();
        let non_auth_endpoint_impls: TokenStream = metadata
            .authentication
            .iter()
            .map(|auth| {
                if auth.value != "None" {
                    TokenStream::new()
                } else {
                    let attrs = &auth.attrs;
                    quote! {
                        #( #attrs )*
                        #[automatically_derived]
                        #[cfg(feature = "client")]
                        impl #request_lifetimes #ruma_api::OutgoingNonAuthRequest
                            for Request #request_lifetimes
                            {}

                        #( #attrs )*
                        #[automatically_derived]
                        #[cfg(feature = "server")]
                        impl #ruma_api::IncomingNonAuthRequest for #incoming_request_type {}
                    }
                }
            })
            .collect();

        quote! {
            #[doc = #docs]
            #[derive(Debug, Clone, #ruma_serde::Outgoing, #ruma_serde::_FakeDeriveSerde)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #[incoming_derive(!Deserialize)]
            #( #struct_attributes )*
            pub struct Request #request_generics #request_def

            #non_auth_endpoint_impls

            #request_body_struct
            #request_query_struct

            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #request_lifetimes #ruma_api::OutgoingRequest for Request #request_lifetimes {
                type EndpointError = #error_ty;
                type IncomingResponse = <Response as #ruma_serde::Outgoing>::Incoming;

                const METADATA: #ruma_api::Metadata = self::METADATA;

                fn try_into_http_request(
                    self,
                    base_url: &::std::primitive::str,
                    access_token: ::std::option::Option<&str>,
                ) -> ::std::result::Result<
                    #http::Request<Vec<u8>>,
                    #ruma_api::error::IntoHttpError,
                > {
                    let metadata = self::METADATA;

                    let mut req_builder = #http::Request::builder()
                        .method(#http::Method::#method)
                        .uri(::std::format!(
                            "{}{}{}",
                            base_url.strip_suffix('/').unwrap_or(base_url),
                            #request_path_string,
                            #request_query_string,
                        ))
                        .header(
                            #ruma_api::exports::http::header::CONTENT_TYPE,
                            "application/json",
                        );

                    let mut req_headers = req_builder
                        .headers_mut()
                        .expect("`http::RequestBuilder` is in unusable state");

                    #header_kvs

                    let http_request = req_builder.body(#request_body)?;

                    Ok(http_request)
                }
            }

            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_api::IncomingRequest for #incoming_request_type {
                type EndpointError = #error_ty;
                type OutgoingResponse = Response;

                const METADATA: #ruma_api::Metadata = self::METADATA;

                fn try_from_http_request<T: #bytes::Buf>(
                    request: #http::Request<T>
                ) -> ::std::result::Result<Self, #ruma_api::error::FromHttpRequestError> {
                    if request.method() != #http::Method::#method {
                        return Err(#ruma_api::error::FromHttpRequestError::MethodMismatch {
                            expected: #http::Method::#method,
                            received: request.method().clone(),
                        });
                    }

                    #parse_request_path
                    #parse_query
                    #parse_headers

                    #extract_body
                    #parse_body

                    Ok(Self {
                        #path_vars
                        #query_vars
                        #header_vars
                        #body_vars
                    })
                }
            }
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
    pub fn new(kind: RequestFieldKind, field: Field, header: Option<Ident>) -> Self {
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
    pub fn kind(&self) -> RequestFieldKind {
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
    pub fn is_body(&self) -> bool {
        self.kind() == RequestFieldKind::Body
    }

    /// Whether or not this request field is a header kind.
    pub fn is_header(&self) -> bool {
        self.kind() == RequestFieldKind::Header
    }

    /// Whether or not this request field is a newtype body kind.
    pub fn is_newtype_body(&self) -> bool {
        self.kind() == RequestFieldKind::NewtypeBody
    }

    /// Whether or not this request field is a path kind.
    pub fn is_path(&self) -> bool {
        self.kind() == RequestFieldKind::Path
    }

    /// Whether or not this request field is a query string kind.
    pub fn is_query(&self) -> bool {
        self.kind() == RequestFieldKind::Query
    }

    /// Return the contained field if this request field is a body kind.
    pub fn as_body_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::Body)
    }

    /// Return the contained field if this request field is a body kind.
    pub fn as_newtype_body_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::NewtypeBody)
    }

    /// Return the contained field if this request field is a raw body kind.
    pub fn as_newtype_raw_body_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::NewtypeRawBody)
    }

    /// Return the contained field if this request field is a query kind.
    pub fn as_query_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::Query)
    }

    /// Return the contained field if this request field is a query map kind.
    pub fn as_query_map_field(&self) -> Option<&Field> {
        self.field_of_kind(RequestFieldKind::QueryMap)
    }

    /// Gets the inner `Field` value.
    pub fn field(&self) -> &Field {
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
    pub fn field_of_kind(&self, kind: RequestFieldKind) -> Option<&Field> {
        if self.kind() == kind {
            Some(self.field())
        } else {
            None
        }
    }
}

/// The types of fields that a request can have, without their values.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestFieldKind {
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
