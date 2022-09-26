use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    DeriveInput, Field, Generics, Ident, Lifetime, LitStr, Token, Type,
};

use super::{
    attribute::{DeriveRequestMeta, RequestMeta},
    auth_scheme::AuthScheme,
    util::collect_lifetime_idents,
};
use crate::util::import_ruma_common;

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
            let ty = &f.inner.ty;

            match &f.kind {
                RequestFieldKind::Header(_) => collect_lifetime_idents(&mut lifetimes.header, ty),
                RequestFieldKind::Body => collect_lifetime_idents(&mut lifetimes.body, ty),
                RequestFieldKind::NewtypeBody => collect_lifetime_idents(&mut lifetimes.body, ty),
                RequestFieldKind::RawBody => collect_lifetime_idents(&mut lifetimes.body, ty),
                RequestFieldKind::Path => collect_lifetime_idents(&mut lifetimes.path, ty),
                RequestFieldKind::Query => collect_lifetime_idents(&mut lifetimes.query, ty),
                RequestFieldKind::QueryMap => collect_lifetime_idents(&mut lifetimes.query, ty),
            }

            Ok(f)
        })
        .collect::<syn::Result<_>>()?;

    let mut authentication = None;
    let mut error_ty = None;
    let mut method = None;
    let mut unstable_path = None;
    let mut r0_path = None;
    let mut stable_path = None;

    for attr in input.attrs {
        if !attr.path.is_ident("ruma_api") {
            continue;
        }

        let metas =
            attr.parse_args_with(Punctuated::<DeriveRequestMeta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                DeriveRequestMeta::Authentication(t) => authentication = Some(parse_quote!(#t)),
                DeriveRequestMeta::Method(t) => method = Some(parse_quote!(#t)),
                DeriveRequestMeta::ErrorTy(t) => error_ty = Some(t),
                DeriveRequestMeta::UnstablePath(s) => unstable_path = Some(s),
                DeriveRequestMeta::R0Path(s) => r0_path = Some(s),
                DeriveRequestMeta::StablePath(s) => stable_path = Some(s),
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
        unstable_path,
        r0_path,
        stable_path,
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
    unstable_path: Option<LitStr>,
    r0_path: Option<LitStr>,
    stable_path: Option<LitStr>,
    error_ty: Type,
}

impl Request {
    fn body_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter_map(RequestField::as_body_field)
    }

    fn has_body_fields(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(&f.kind, RequestFieldKind::Body | RequestFieldKind::NewtypeBody))
    }

    fn has_newtype_body(&self) -> bool {
        self.fields.iter().any(|f| matches!(&f.kind, RequestFieldKind::NewtypeBody))
    }

    fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(&f.kind, RequestFieldKind::Header(_)))
    }

    fn has_path_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(&f.kind, RequestFieldKind::Path))
    }

    fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(&f.kind, RequestFieldKind::Query))
    }

    fn has_lifetimes(&self) -> bool {
        !(self.lifetimes.body.is_empty()
            && self.lifetimes.path.is_empty()
            && self.lifetimes.query.is_empty()
            && self.lifetimes.header.is_empty())
    }

    fn header_fields(&self) -> impl Iterator<Item = (&Field, &Ident)> {
        self.fields.iter().filter_map(RequestField::as_header_field)
    }

    fn path_fields_ordered(&self) -> impl Iterator<Item = &Field> {
        let map: BTreeMap<String, &Field> = self
            .fields
            .iter()
            .filter_map(RequestField::as_path_field)
            .map(|f| (f.ident.as_ref().unwrap().to_string(), f))
            .collect();

        self.stable_path
            .as_ref()
            .or(self.r0_path.as_ref())
            .or(self.unstable_path.as_ref())
            .expect("one of the paths to be defined")
            .value()
            .split('/')
            .filter_map(|s| {
                s.strip_prefix(':')
                    .map(|s| *map.get(s).expect("path args have already been checked"))
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_raw_body_field)
    }

    fn query_map_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_query_map_field)
    }

    fn expand_all(&self) -> TokenStream {
        let ruma_common = import_ruma_common();
        let ruma_macros = quote! { #ruma_common::exports::ruma_macros };
        let serde = quote! { #ruma_common::exports::serde };

        let request_body_struct = self.has_body_fields().then(|| {
            let serde_attr = self.has_newtype_body().then(|| quote! { #[serde(transparent)] });
            let fields = self.fields.iter().filter_map(RequestField::as_body_field);

            // Though we don't track the difference between newtype body and body
            // for lifetimes, the outer check and the macro failing if it encounters
            // an illegal combination of field attributes, is enough to guarantee
            // `body_lifetimes` correctness.
            let lifetimes = &self.lifetimes.body;
            let derive_deserialize = lifetimes.is_empty().then(|| quote! { #serde::Deserialize });

            quote! {
                /// Data in the request body.
                #[cfg(any(feature = "client", feature = "server"))]
                #[derive(Debug, #ruma_macros::_FakeDeriveRumaApi, #ruma_macros::_FakeDeriveSerde)]
                #[cfg_attr(feature = "client", derive(#serde::Serialize))]
                #[cfg_attr(
                    feature = "server",
                    derive(#ruma_common::serde::Incoming, #derive_deserialize)
                )]
                #serde_attr
                struct RequestBody< #(#lifetimes),* > { #(#fields),* }
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
            let lifetimes = &self.lifetimes.query;
            let derive_deserialize = lifetimes.is_empty().then(|| quote! { #serde::Deserialize });

            quote! {
                /// Data in the request's query string.
                #[cfg(any(feature = "client", feature = "server"))]
                #[derive(Debug, #ruma_macros::_FakeDeriveRumaApi, #ruma_macros::_FakeDeriveSerde)]
                #[cfg_attr(feature = "client", derive(#serde::Serialize))]
                #[cfg_attr(
                    feature = "server",
                    derive(#ruma_common::serde::Incoming, #derive_deserialize)
                )]
                struct RequestQuery< #(#lifetimes),* > #def
            }
        });

        let outgoing_request_impl = self.expand_outgoing(&ruma_common);
        let incoming_request_impl = self.expand_incoming(&ruma_common);

        quote! {
            #request_body_struct
            #request_query_struct

            #outgoing_request_impl
            #incoming_request_impl
        }
    }

    pub(super) fn check(&self) -> syn::Result<()> {
        // TODO: highlight problematic fields

        let path_fields: Vec<_> =
            self.fields.iter().filter_map(RequestField::as_path_field).collect();

        self.check_path(&path_fields, self.unstable_path.as_ref())?;
        self.check_path(&path_fields, self.r0_path.as_ref())?;
        self.check_path(&path_fields, self.stable_path.as_ref())?;

        let newtype_body_fields = self.fields.iter().filter(|f| {
            matches!(&f.kind, RequestFieldKind::NewtypeBody | RequestFieldKind::RawBody)
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
            self.fields.iter().filter(|f| matches!(&f.kind, RequestFieldKind::QueryMap));
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

        let has_body_fields = self.fields.iter().any(|f| matches!(&f.kind, RequestFieldKind::Body));
        let has_query_fields =
            self.fields.iter().any(|f| matches!(&f.kind, RequestFieldKind::Query));

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

    fn check_path(&self, fields: &[&Field], path: Option<&LitStr>) -> syn::Result<()> {
        let path = if let Some(lit) = path { lit } else { return Ok(()) };

        let path_args: Vec<_> = path
            .value()
            .split('/')
            .filter_map(|s| s.strip_prefix(':').map(str::to_string))
            .collect();

        let field_map: BTreeMap<_, _> =
            fields.iter().map(|&f| (f.ident.as_ref().unwrap().to_string(), f)).collect();

        // test if all macro fields exist in the path
        for (name, field) in field_map.iter() {
            if !path_args.contains(name) {
                return Err({
                    let mut err = syn::Error::new_spanned(
                        field,
                        "this path argument field is not defined in...",
                    );
                    err.combine(syn::Error::new_spanned(path, "...this path."));
                    err
                });
            }
        }

        // test if all path fields exists in macro fields
        for arg in &path_args {
            if !field_map.contains_key(arg) {
                return Err(syn::Error::new_spanned(
                    path,
                    format!(
                        "a corresponding request path argument field for \"{}\" does not exist",
                        arg
                    ),
                ));
            }
        }

        Ok(())
    }
}

/// A field of the request struct.
struct RequestField {
    inner: Field,
    kind: RequestFieldKind,
}

/// The kind of a request field.
enum RequestFieldKind {
    /// JSON data in the body of the request.
    Body,

    /// Data in an HTTP header.
    Header(Ident),

    /// A specific data type in the body of the request.
    NewtypeBody,

    /// Arbitrary bytes in the body of the request.
    RawBody,

    /// Data that appears in the URL path.
    Path,

    /// Data that appears in the query string.
    Query,

    /// Data that appears in the query string as dynamic key-value pairs.
    QueryMap,
}

impl RequestField {
    /// Creates a new `RequestField`.
    fn new(inner: Field, kind_attr: Option<RequestMeta>) -> Self {
        let kind = match kind_attr {
            Some(RequestMeta::NewtypeBody) => RequestFieldKind::NewtypeBody,
            Some(RequestMeta::RawBody) => RequestFieldKind::RawBody,
            Some(RequestMeta::Path) => RequestFieldKind::Path,
            Some(RequestMeta::Query) => RequestFieldKind::Query,
            Some(RequestMeta::QueryMap) => RequestFieldKind::QueryMap,
            Some(RequestMeta::Header(header)) => RequestFieldKind::Header(header),
            None => RequestFieldKind::Body,
        };

        Self { inner, kind }
    }

    /// Return the contained field if this request field is a body kind.
    pub fn as_body_field(&self) -> Option<&Field> {
        match &self.kind {
            RequestFieldKind::Body | RequestFieldKind::NewtypeBody => Some(&self.inner),
            _ => None,
        }
    }

    /// Return the contained field if this request field is a raw body kind.
    pub fn as_raw_body_field(&self) -> Option<&Field> {
        match &self.kind {
            RequestFieldKind::RawBody => Some(&self.inner),
            _ => None,
        }
    }

    /// Return the contained field if this request field is a path kind.
    pub fn as_path_field(&self) -> Option<&Field> {
        match &self.kind {
            RequestFieldKind::Path => Some(&self.inner),
            _ => None,
        }
    }

    /// Return the contained field if this request field is a query kind.
    pub fn as_query_field(&self) -> Option<&Field> {
        match &self.kind {
            RequestFieldKind::Query => Some(&self.inner),
            _ => None,
        }
    }

    /// Return the contained field if this request field is a query map kind.
    pub fn as_query_map_field(&self) -> Option<&Field> {
        match &self.kind {
            RequestFieldKind::QueryMap => Some(&self.inner),
            _ => None,
        }
    }

    /// Return the contained field and header ident if this request field is a header kind.
    pub fn as_header_field(&self) -> Option<(&Field, &Ident)> {
        match &self.kind {
            RequestFieldKind::Header(header_name) => Some((&self.inner, header_name)),
            _ => None,
        }
    }
}

impl TryFrom<Field> for RequestField {
    type Error = syn::Error;

    fn try_from(mut field: Field) -> syn::Result<Self> {
        let (mut api_attrs, attrs) =
            field.attrs.into_iter().partition::<Vec<_>, _>(|attr| attr.path.is_ident("ruma_api"));
        field.attrs = attrs;

        let kind_attr = match api_attrs.as_slice() {
            [] => None,
            [_] => Some(api_attrs.pop().unwrap().parse_args::<RequestMeta>()?),
            _ => {
                return Err(syn::Error::new_spanned(
                    &api_attrs[1],
                    "multiple field kind attribute found, there can only be one",
                ));
            }
        };

        Ok(RequestField::new(field, kind_attr))
    }
}

impl Parse for RequestField {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.call(Field::parse_named)?.try_into()
    }
}

impl ToTokens for RequestField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens);
    }
}
