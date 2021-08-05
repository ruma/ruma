use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
    mem,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    DeriveInput, Field, Generics, Ident, Lifetime, Lit, LitStr, Token, Type,
};

use crate::{
    attribute::{Meta, MetaNameValue, MetaValue},
    auth_scheme::AuthScheme,
    util::{collect_lifetime_idents, import_ruma_api},
};

mod incoming;
mod outgoing;

pub fn expand_derive_request(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        syn::Data::Struct(s) => s.fields,
        _ => panic!("This derive macro only works on structs"),
    };

    let mut lifetimes = RequestLifetimes::default();
    let fields = fields
        .into_iter()
        .map(|f| {
            let f = RequestField::try_from(f)?;
            let ty = &f.field().ty;

            match &f {
                RequestField::Header(..) => collect_lifetime_idents(&mut lifetimes.header, ty),
                RequestField::Body(_) => collect_lifetime_idents(&mut lifetimes.body, ty),
                RequestField::NewtypeBody(_) => collect_lifetime_idents(&mut lifetimes.body, ty),
                RequestField::NewtypeRawBody(_) => collect_lifetime_idents(&mut lifetimes.body, ty),
                RequestField::Path(_) => collect_lifetime_idents(&mut lifetimes.path, ty),
                RequestField::Query(_) => collect_lifetime_idents(&mut lifetimes.query, ty),
                RequestField::QueryMap(_) => collect_lifetime_idents(&mut lifetimes.query, ty),
            }

            Ok(f)
        })
        .collect::<syn::Result<_>>()?;

    let mut authentication = None;
    let mut error_ty = None;
    let mut method = None;
    let mut path = None;

    for attr in input.attrs {
        if !attr.path.is_ident("ruma_api") {
            continue;
        }

        let meta = attr.parse_args_with(Punctuated::<_, Token![,]>::parse_terminated)?;
        for MetaNameValue { name, value } in meta {
            match value {
                MetaValue::Type(t) if name == "authentication" => {
                    authentication = Some(parse_quote!(#t));
                }
                MetaValue::Type(t) if name == "method" => {
                    method = Some(parse_quote!(#t));
                }
                MetaValue::Type(t) if name == "error_ty" => {
                    error_ty = Some(t);
                }
                MetaValue::Lit(Lit::Str(s)) if name == "path" => {
                    path = Some(s);
                }
                _ => unreachable!("invalid ruma_api({}) attribute", name),
            }
        }
    }

    let request = Request {
        ident: input.ident,
        generics: input.generics,
        fields,
        lifetimes,
        authentication: authentication.expect("missing authentication attribute"),
        method: method.expect("missing method attribute"),
        path: path.expect("missing path attribute"),
        error_ty: error_ty.expect("missing error_ty attribute"),
    };

    request.check()?;
    Ok(request.expand_all())
}

#[derive(Default)]
struct RequestLifetimes {
    pub body: BTreeSet<Lifetime>,
    pub path: BTreeSet<Lifetime>,
    pub query: BTreeSet<Lifetime>,
    pub header: BTreeSet<Lifetime>,
}

struct Request {
    ident: Ident,
    generics: Generics,
    lifetimes: RequestLifetimes,
    fields: Vec<RequestField>,

    authentication: AuthScheme,
    method: Ident,
    path: LitStr,
    error_ty: Type,
}

impl Request {
    fn body_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter_map(RequestField::as_body_field)
    }

    fn query_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter_map(RequestField::as_query_field)
    }

    fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, RequestField::Body(..)))
    }

    fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, RequestField::Header(..)))
    }

    fn has_path_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, RequestField::Path(..)))
    }

    fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, RequestField::Query(..)))
    }

    fn has_lifetimes(&self) -> bool {
        !(self.lifetimes.body.is_empty()
            && self.lifetimes.path.is_empty()
            && self.lifetimes.query.is_empty()
            && self.lifetimes.header.is_empty())
    }

    fn header_fields(&self) -> impl Iterator<Item = &RequestField> {
        self.fields.iter().filter(|f| matches!(f, RequestField::Header(..)))
    }

    fn path_field_count(&self) -> usize {
        self.fields.iter().filter(|f| matches!(f, RequestField::Path(..))).count()
    }

    fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_newtype_body_field)
    }

    fn newtype_raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_newtype_raw_body_field)
    }

    fn query_map_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_query_map_field)
    }

    fn expand_all(&self) -> TokenStream {
        let ruma_api = import_ruma_api();
        let ruma_api_macros = quote! { #ruma_api::exports::ruma_api_macros };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };

        let request_body_def = if let Some(body_field) = self.newtype_body_field() {
            let field = Field { ident: None, colon_token: None, ..body_field.clone() };
            Some(quote! { (#field); })
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter_map(RequestField::as_body_field);
            Some(quote! { { #(#fields),* } })
        } else {
            None
        };

        let request_body_struct = request_body_def.map(|def| {
            // Though we don't track the difference between newtype body and body
            // for lifetimes, the outer check and the macro failing if it encounters
            // an illegal combination of field attributes, is enough to guarantee
            // `body_lifetimes` correctness.
            let (derive_deserialize, generics) = if self.lifetimes.body.is_empty() {
                (quote! { #serde::Deserialize }, TokenStream::new())
            } else {
                let lifetimes = &self.lifetimes.body;
                (TokenStream::new(), quote! { < #(#lifetimes),* > })
            };

            quote! {
                /// Data in the request body.
                #[derive(
                    Debug,
                    #ruma_api_macros::_FakeDeriveRumaApi,
                    #ruma_serde::Outgoing,
                    #serde::Serialize,
                    #derive_deserialize
                )]
                struct RequestBody #generics #def
            }
        });

        let request_query_def = if let Some(f) = self.query_map_field() {
            let field = Field { ident: None, colon_token: None, ..f.clone() };
            Some(quote! { (#field); })
        } else if self.has_query_fields() {
            let fields = self.fields.iter().filter_map(RequestField::as_query_field);
            Some(quote! { { #(#fields),* } })
        } else {
            None
        };

        let request_query_struct = request_query_def.map(|def| {
            let (derive_deserialize, generics) = if self.lifetimes.query.is_empty() {
                (quote! { #serde::Deserialize }, TokenStream::new())
            } else {
                let lifetimes = &self.lifetimes.query;
                (TokenStream::new(), quote! { < #(#lifetimes),* > })
            };

            quote! {
                /// Data in the request's query string.
                #[derive(
                    Debug,
                    #ruma_api_macros::_FakeDeriveRumaApi,
                    #ruma_serde::Outgoing,
                    #serde::Serialize,
                    #derive_deserialize
                )]
                struct RequestQuery #generics #def
            }
        });

        let outgoing_request_impl = self.expand_outgoing(&ruma_api);
        let incoming_request_impl = self.expand_incoming(&ruma_api);

        quote! {
            #request_body_struct
            #request_query_struct

            #outgoing_request_impl
            #incoming_request_impl
        }
    }

    pub(super) fn check(&self) -> syn::Result<()> {
        // TODO: highlight problematic fields

        let newtype_body_fields = self.fields.iter().filter(|field| {
            matches!(field, RequestField::NewtypeBody(_) | RequestField::NewtypeRawBody(_))
        });

        let has_newtype_body_field = match newtype_body_fields.count() {
            0 => false,
            1 => true,
            _ => {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "Can't have more than one newtype body field",
                ))
            }
        };

        let query_map_fields =
            self.fields.iter().filter(|f| matches!(f, RequestField::QueryMap(_)));
        let has_query_map_field = match query_map_fields.count() {
            0 => false,
            1 => true,
            _ => {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "Can't have more than one query_map field",
                ))
            }
        };

        let has_body_fields = self.body_fields().count() > 0;
        let has_query_fields = self.query_fields().count() > 0;

        if has_newtype_body_field && has_body_fields {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        if has_query_map_field && has_query_fields {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "Can't have both a query map field and regular query fields",
            ));
        }

        // TODO when/if `&[(&str, &str)]` is supported remove this
        if has_query_map_field && !self.lifetimes.query.is_empty() {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "Lifetimes are not allowed for query_map fields",
            ));
        }

        if self.method == "GET" && (has_body_fields || has_newtype_body_field) {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "GET endpoints can't have body fields",
            ));
        }

        Ok(())
    }
}

/// The types of fields that a request can have.
enum RequestField {
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

impl TryFrom<Field> for RequestField {
    type Error = syn::Error;

    fn try_from(mut field: Field) -> syn::Result<Self> {
        let mut field_kind = None;
        let mut header = None;

        for attr in mem::take(&mut field.attrs) {
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
                    "body" => RequestFieldKind::Body,
                    "raw_body" => RequestFieldKind::NewtypeRawBody,
                    "path" => RequestFieldKind::Path,
                    "query" => RequestFieldKind::Query,
                    "query_map" => RequestFieldKind::QueryMap,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "Invalid #[ruma_api] argument, expected one of \
                            `body`, `raw_body`, `path`, `query`, `query_map`",
                        ));
                    }
                },
                Meta::NameValue(MetaNameValue { name, value }) => {
                    if name != "header" {
                        return Err(syn::Error::new_spanned(
                            name,
                            "Invalid #[ruma_api] argument with value, expected `header`",
                        ));
                    }

                    header = Some(value);
                    RequestFieldKind::Header
                }
            });
        }

        Ok(RequestField::new(field_kind.unwrap_or(RequestFieldKind::Body), field, header))
    }
}

impl Parse for RequestField {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.call(Field::parse_named)?.try_into()
    }
}

impl ToTokens for RequestField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.field().to_tokens(tokens)
    }
}

/// The types of fields that a request can have, without their values.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestFieldKind {
    Body,
    Header,
    NewtypeBody,
    NewtypeRawBody,
    Path,
    Query,
    QueryMap,
}
