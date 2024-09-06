use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Field, Generics, Ident, ItemStruct, Token, Type,
};

use super::{
    attribute::{DeriveRequestMeta, RequestMeta},
    ensure_feature_presence,
};
use crate::util::{import_ruma_common, PrivateField};

mod incoming;
mod outgoing;

pub fn expand_request(attr: RequestAttr, item: ItemStruct) -> TokenStream {
    let ruma_common = import_ruma_common();
    let ruma_macros = quote! { #ruma_common::exports::ruma_macros };

    let maybe_feature_error = ensure_feature_presence().map(syn::Error::to_compile_error);

    let error_ty = attr.0.first().map_or_else(
        || quote! { #ruma_common::api::error::MatrixError },
        |DeriveRequestMeta::Error(ty)| quote! { #ty },
    );

    cfg_if! {
        if #[cfg(feature = "__internal_macro_expand")] {
            use syn::parse_quote;

            let mut derive_input = item.clone();
            derive_input.attrs.push(parse_quote! { #[ruma_api(error = #error_ty)] });
            crate::util::cfg_expand_struct(&mut derive_input);

            let extra_derive = quote! { #ruma_macros::_FakeDeriveRumaApi };
            let ruma_api_attribute = quote! {};
            let request_impls =
                expand_derive_request(derive_input).unwrap_or_else(syn::Error::into_compile_error);
        } else {
            let extra_derive = quote! { #ruma_macros::Request };
            let ruma_api_attribute = quote! { #[ruma_api(error = #error_ty)] };
            let request_impls = quote! {};
        }
    }

    quote! {
        #maybe_feature_error

        #[derive(Clone, Debug, #ruma_common::serde::_FakeDeriveSerde, #extra_derive)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        #ruma_api_attribute
        #item

        #request_impls
    }
}

pub struct RequestAttr(Punctuated<DeriveRequestMeta, Token![,]>);

impl Parse for RequestAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Punctuated::<DeriveRequestMeta, Token![,]>::parse_terminated(input).map(Self)
    }
}

pub fn expand_derive_request(input: ItemStruct) -> syn::Result<TokenStream> {
    let fields =
        input.fields.into_iter().map(RequestField::try_from).collect::<syn::Result<_>>()?;

    let mut error_ty = None;

    for attr in input.attrs {
        if !attr.path().is_ident("ruma_api") {
            continue;
        }

        let metas =
            attr.parse_args_with(Punctuated::<DeriveRequestMeta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                DeriveRequestMeta::Error(t) => error_ty = Some(t),
            }
        }
    }

    let request = Request {
        ident: input.ident,
        generics: input.generics,
        fields,
        error_ty: error_ty.expect("missing error_ty attribute"),
    };

    let ruma_common = import_ruma_common();
    let test = request.check(&ruma_common)?;
    let types_impls = request.expand_all(&ruma_common);

    Ok(quote! {
        #types_impls

        #[allow(deprecated)]
        #[cfg(tests)]
        mod __request {
            #test
        }
    })
}

struct Request {
    ident: Ident,
    generics: Generics,
    fields: Vec<RequestField>,

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

    fn header_fields(&self) -> impl Iterator<Item = (&Field, &Ident)> {
        self.fields.iter().filter_map(RequestField::as_header_field)
    }

    fn path_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter_map(RequestField::as_path_field)
    }

    fn raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_raw_body_field)
    }

    fn query_all_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(RequestField::as_query_all_field)
    }

    fn expand_all(&self, ruma_common: &TokenStream) -> TokenStream {
        let ruma_macros = quote! { #ruma_common::exports::ruma_macros };
        let serde = quote! { #ruma_common::exports::serde };

        let request_body_struct = self.has_body_fields().then(|| {
            let serde_attr = self.has_newtype_body().then(|| quote! { #[serde(transparent)] });
            let fields =
                self.fields.iter().filter_map(RequestField::as_body_field).map(PrivateField);

            quote! {
                /// Data in the request body.
                #[cfg(any(feature = "client", feature = "server"))]
                #[derive(Debug, #ruma_macros::_FakeDeriveRumaApi, #ruma_macros::_FakeDeriveSerde)]
                #[cfg_attr(feature = "client", derive(#serde::Serialize))]
                #[cfg_attr(feature = "server", derive(#serde::Deserialize))]
                #serde_attr
                struct RequestBody { #(#fields),* }
            }
        });

        let request_query_def = if let Some(f) = self.query_all_field() {
            let field = Field { ident: None, colon_token: None, ..f.clone() };
            let field = PrivateField(&field);
            Some(quote! { (#field); })
        } else if self.has_query_fields() {
            let fields =
                self.fields.iter().filter_map(RequestField::as_query_field).map(PrivateField);
            Some(quote! { { #(#fields),* } })
        } else {
            None
        };

        let request_query_struct = request_query_def.map(|def| {
            quote! {
                /// Data in the request's query string.
                #[cfg(any(feature = "client", feature = "server"))]
                #[derive(Debug, #ruma_macros::_FakeDeriveRumaApi, #ruma_macros::_FakeDeriveSerde)]
                #[cfg_attr(feature = "client", derive(#serde::Serialize))]
                #[cfg_attr(feature = "server", derive(#serde::Deserialize))]
                struct RequestQuery #def
            }
        });

        let outgoing_request_impl = self.expand_outgoing(ruma_common);
        let incoming_request_impl = self.expand_incoming(ruma_common);

        quote! {
            #request_body_struct
            #request_query_struct

            #[allow(deprecated)]
            mod __request_impls {
                use super::*;
                #outgoing_request_impl
                #incoming_request_impl
            }
        }
    }

    pub(super) fn check(&self, ruma_common: &TokenStream) -> syn::Result<TokenStream> {
        let http = quote! { #ruma_common::exports::http };

        // TODO: highlight problematic fields

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

        let query_all_fields =
            self.fields.iter().filter(|f| matches!(&f.kind, RequestFieldKind::QueryAll));
        let has_query_all_field = match query_all_fields.count() {
            0 => false,
            1 => true,
            _ => {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "Can't have more than one query_all field",
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

        if has_query_all_field && has_query_fields {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "Can't have both a query_all field and regular query fields",
            ));
        }

        let path_fields = self.path_fields().map(|f| f.ident.as_ref().unwrap().to_string());
        let mut tests = quote! {
            #[::std::prelude::v1::test]
            #[allow(deprecated)]
            fn path_parameters() {
                let path_params = METADATA._path_parameters();
                let request_path_fields: &[&::std::primitive::str] = &[#(#path_fields),*];
                ::std::assert_eq!(
                    path_params, request_path_fields,
                    "Path parameters must match the `Request`'s `#[ruma_api(path)]` fields"
                );
            }
        };

        if has_body_fields || has_newtype_body_field {
            tests.extend(quote! {
                #[::std::prelude::v1::test]
                fn request_is_not_get() {
                    ::std::assert_ne!(
                        METADATA.method, #http::Method::GET,
                        "GET endpoints can't have body fields",
                    );
                }
            });
        }

        Ok(tests)
    }
}

/// A field of the request struct.
pub(super) struct RequestField {
    pub(super) inner: Field,
    pub(super) kind: RequestFieldKind,
}

/// The kind of a request field.
pub(super) enum RequestFieldKind {
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

    /// Data that represents all the query string as a single type.
    QueryAll,
}

impl RequestField {
    /// Creates a new `RequestField`.
    fn new(inner: Field, kind_attr: Option<RequestMeta>) -> Self {
        let kind = match kind_attr {
            Some(RequestMeta::NewtypeBody) => RequestFieldKind::NewtypeBody,
            Some(RequestMeta::RawBody) => RequestFieldKind::RawBody,
            Some(RequestMeta::Path) => RequestFieldKind::Path,
            Some(RequestMeta::Query) => RequestFieldKind::Query,
            Some(RequestMeta::QueryAll) => RequestFieldKind::QueryAll,
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

    /// Return the contained field if this request field is a query all kind.
    pub fn as_query_all_field(&self) -> Option<&Field> {
        match &self.kind {
            RequestFieldKind::QueryAll => Some(&self.inner),
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
            field.attrs.into_iter().partition::<Vec<_>, _>(|attr| attr.path().is_ident("ruma_api"));
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
