//! Types and implementations to parse the request macro's input.

use proc_macro2::Span;
use syn::{meta::ParseNestedMeta, parse_quote};

use super::Response;
use crate::{
    api::{Body, Headers},
    util::{ParseNestedMetaExt, RumaCommon, TypeExt},
};

impl Response {
    /// Validate this response after it is fully parsed.
    fn validate(&self) -> syn::Result<()> {
        if !self.generics.params.is_empty() || self.generics.where_clause.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "the `response` macro doesn't support generic types",
            ));
        }

        self.body.validate()?;

        Ok(())
    }
}

impl TryFrom<syn::ItemStruct> for Response {
    type Error = syn::Error;

    fn try_from(input: syn::ItemStruct) -> Result<Self, Self::Error> {
        // Parse container attributes.
        let mut response_attrs = ResponseAttrs::default();

        for attr in input.attrs {
            if !attr.path().is_ident("ruma_api") {
                continue;
            }

            attr.parse_nested_meta(|meta| response_attrs.try_merge(meta))?;
        }

        let mut response = Response {
            ident: input.ident,
            generics: input.generics,
            headers: Headers::default(),
            body: Body::default(),
            error_ty: response_attrs
                .error_ty
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `error` attribute"))?,
            status: response_attrs
                .status
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `status` attribute"))?,
        };

        response.body.set_manual_serde(response_attrs.manual_body_serde);

        // Parse struct fields.
        for field in input.fields {
            let ResponseField { inner: field, kind } = field.try_into()?;

            match kind {
                ResponseFieldKind::Body => response.body.push_json_field(field)?,
                ResponseFieldKind::NewtypeBody => response.body.set_json_all(field)?,
                ResponseFieldKind::RawBody => response.body.set_raw(field)?,
                ResponseFieldKind::Header { name } => {
                    response.headers.insert(name, field)?;
                }
            }
        }

        response.validate()?;

        Ok(response)
    }
}

/// Attributes on the response struct.
#[derive(Default)]
pub(crate) struct ResponseAttrs {
    /// The type used for the `EndpointError` associated type on `OutgoingResponse` and
    /// `IncomingResponse` implementations.
    error_ty: Option<syn::Type>,

    /// The HTTP status code to use for the response.
    status: Option<syn::Ident>,

    /// Whether the response implements `Serialize` and `Deserialize` manually.
    pub(super) manual_body_serde: bool,
}

impl ResponseAttrs {
    /// Set the error type of this `ResponseAttrs`.
    ///
    /// Returns an error if the error type is already set.
    fn set_error_ty(&mut self, error_ty: syn::Type) -> syn::Result<()> {
        if self.error_ty.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "cannot have multiple values for `error` response attribute",
            ));
        }

        self.error_ty = Some(error_ty);
        Ok(())
    }

    /// Set the HTTP status code of this `ResponseAttrs`.
    ///
    /// Returns an error if the status code is already set.
    fn set_status(&mut self, status: syn::Ident) -> syn::Result<()> {
        if self.status.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "cannot have multiple values for `status` response attribute",
            ));
        }

        self.status = Some(status);
        Ok(())
    }

    /// Set that the response implements `Serialize` and `Deserialize` manually.
    ///
    /// Returns an error if it is already set.
    fn set_manual_body_serde(&mut self) -> syn::Result<()> {
        if self.manual_body_serde {
            return Err(syn::Error::new(
                Span::call_site(),
                "cannot have multiple `manual_body_serde` response attributes",
            ));
        }

        self.manual_body_serde = true;
        Ok(())
    }

    /// Try to parse the given meta item and merge it into this `RequestAttrs`.
    ///
    /// Returns an error if parsing the meta item fails, or if it sets a field that was already set.
    pub(crate) fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.path.is_ident("error") {
            return self.set_error_ty(meta.value()?.parse()?);
        }

        if meta.path.is_ident("status") {
            return self.set_status(meta.value()?.parse()?);
        }

        if meta.path.is_ident("manual_body_serde") {
            return self.set_manual_body_serde();
        }

        Err(meta.error("unsupported `response` attribute"))
    }

    /// The error type that was set on the response, or the default value which is `MatrixError`.
    pub(super) fn error_ty_or_default(&self, ruma_common: &RumaCommon) -> syn::Type {
        self.error_ty
            .clone()
            .unwrap_or_else(|| parse_quote! { #ruma_common::api::error::MatrixError })
    }

    /// The HTTP status code that was set on the response, or the default value which is `OK`.
    pub(super) fn status_or_default(&self) -> syn::Ident {
        self.status.clone().unwrap_or_else(|| parse_quote! { OK })
    }
}

/// A parsed field of a response struct.
struct ResponseField {
    /// The field with the `ruma_api` attributes stripped.
    inner: syn::Field,

    /// The kind of field.
    kind: ResponseFieldKind,
}

impl ResponseField {
    /// Set the kind of this `ResponseField`.
    ///
    /// Returns an error if the kind was already set.
    fn set_kind(&mut self, kind: ResponseFieldKind) -> syn::Result<()> {
        if !matches!(self.kind, ResponseFieldKind::Body) {
            return Err(syn::Error::new_spanned(
                &self.inner,
                "multiple request field kind attributes found, there can only be one",
            ));
        }

        self.kind = kind;
        Ok(())
    }

    /// Try to merge the values of the attributes in the given meta in this `ResponseField`.
    ///
    /// Returns an error if parsing the meta fails or a value is set twice.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if let Some(kind) = ResponseFieldKind::try_from_meta(&meta)? {
            return self.set_kind(kind);
        }

        Err(meta.error("unsupported `ruma_api` field attribute"))
    }
}

impl TryFrom<syn::Field> for ResponseField {
    type Error = syn::Error;

    fn try_from(inner: syn::Field) -> syn::Result<Self> {
        if inner.ty.has_lifetime() {
            return Err(syn::Error::new_spanned(
                inner,
                "lifetimes on response fields cannot be supported until GAT are stable",
            ));
        }

        let mut field = ResponseField { inner, kind: Default::default() };

        let api_attrs = field
            .inner
            .attrs
            .extract_if(.., |attr| attr.path().is_ident("ruma_api"))
            .collect::<Vec<_>>();

        for attr in api_attrs {
            attr.parse_nested_meta(|meta| field.try_merge(meta))?;
        }

        Ok(field)
    }
}

/// The kind of a response field.
#[derive(Default)]
enum ResponseFieldKind {
    /// Part of the JSON data in the body of the response.
    #[default]
    Body,

    /// The full JSON data in the body of the response.
    NewtypeBody,

    /// Arbitrary bytes in the body of the response.
    RawBody,

    /// Data in an HTTP header.
    Header {
        /// The name of the header, as a constant from `http::header`.
        name: syn::Ident,
    },
}

impl ResponseFieldKind {
    /// Try to convert the given meta into a `ResponseFieldKind`.
    ///
    /// Returns `Ok(Some(kind))` if the meta matches one of the variants and parsing it succeeds,
    /// `Ok(None)` if the meta doesn't match one of the variants, and `Err(_)` if the meta matches
    /// one of the variants but parsing it fails.
    fn try_from_meta(meta: &ParseNestedMeta<'_>) -> syn::Result<Option<Self>> {
        let Some(ident) = meta.path.get_ident() else {
            return Ok(None);
        };

        match ident.to_string().as_str() {
            "body" => {
                if meta.has_value() {
                    return Err(meta.error("`body` attribute doesn't expect a value"));
                }

                Ok(Some(Self::NewtypeBody))
            }
            "raw_body" => {
                if meta.has_value() {
                    return Err(meta.error("`raw_body` attribute doesn't expect a value"));
                }

                Ok(Some(Self::RawBody))
            }
            "header" => {
                let name = meta.value()?.parse()?;
                Ok(Some(Self::Header { name }))
            }
            _ => Ok(None),
        }
    }
}
