//! Types and implementations to parse the request macro's input.

use proc_macro2::{Span, TokenStream};
use syn::{meta::ParseNestedMeta, parse_quote};

use super::{Request, RequestBody, RequestHeaders, RequestPath, RequestQuery};
use crate::util::StructFieldExt;

impl Request {
    /// Validate this request after it is fully parsed.
    fn validate(&self) -> syn::Result<()> {
        self.path.validate()?;
        self.body.validate()?;

        Ok(())
    }
}

impl TryFrom<syn::ItemStruct> for Request {
    type Error = syn::Error;

    fn try_from(input: syn::ItemStruct) -> Result<Self, Self::Error> {
        // Parse container attributes.
        let mut request_attrs = RequestAttrs::default();

        for attr in input.attrs {
            if !attr.path().is_ident("ruma_api") {
                continue;
            }

            attr.parse_nested_meta(|meta| request_attrs.try_merge(meta))?;
        }

        let mut request = Request {
            ident: input.ident,
            generics: input.generics,
            headers: RequestHeaders::default(),
            path: RequestPath::default(),
            query: RequestQuery::default(),
            body: RequestBody::default(),
            error_ty: request_attrs
                .error_ty
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `error` attribute"))?,
        };

        // Parse struct fields.
        for field in input.fields {
            let RequestField { inner: field, kind } = field.try_into()?;

            match kind {
                RequestFieldKind::Body => request.body.push_json_field(field)?,
                RequestFieldKind::NewtypeBody => request.body.set_json_all(field)?,
                RequestFieldKind::RawBody => request.body.set_raw(field)?,
                RequestFieldKind::Path => {
                    request.path.push(field);
                }
                RequestFieldKind::Query => request.query.push_field(field)?,
                RequestFieldKind::QueryAll => request.query.set_all(field)?,
                RequestFieldKind::Header { name } => {
                    request.headers.insert(name, field)?;
                }
            }
        }

        request.validate()?;

        Ok(request)
    }
}

impl RequestHeaders {
    /// Insert the given header to this `RequestHeaders`.
    ///
    /// Returns an error if the given header is already set.
    fn insert(&mut self, header: syn::Ident, field: syn::Field) -> syn::Result<()> {
        if self.0.contains_key(&header) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("cannot have multiple values for `{header}` header"),
            ));
        }

        self.0.insert(header, field);

        Ok(())
    }
}

impl RequestPath {
    /// Add the given field to this `RequestPath`.
    fn push(&mut self, field: syn::Field) {
        self.0.push(field);
    }

    /// Validate the fields in this `RequestPath`.
    fn validate(&self) -> syn::Result<()> {
        for field in &self.0 {
            if field.attrs.iter().any(|attr| attr.path().is_ident("cfg")) {
                return Err(syn::Error::new_spanned(
                    field,
                    "`#[cfg]` attribute is not supported on `path` fields",
                ));
            }
        }

        Ok(())
    }
}

impl RequestQuery {
    /// Add the given field to the list of [`RequestQuery::Fields`].
    ///
    /// Returns an error if this is not a [`RequestQuery::None`] or [`RequestQuery::Fields`]
    /// variant.
    fn push_field(&mut self, field: syn::Field) -> syn::Result<()> {
        match self {
            Self::None => {
                *self = Self::Fields(vec![field]);
                Ok(())
            }
            Self::Fields(fields) => {
                fields.push(field);
                Ok(())
            }
            Self::All(_) => Err(syn::Error::new(
                Span::call_site(),
                "cannot have both a `query_all` field and `query` fields",
            )),
        }
    }

    /// Set this as a [`RequestQuery::All`] with the given field.
    ///
    /// Returns an error if this is not a [`RequestQuery::None`].
    fn set_all(&mut self, field: syn::Field) -> syn::Result<()> {
        let error_msg = match self {
            Self::None => {
                *self = Self::All(field);
                return Ok(());
            }
            Self::Fields(_) => "cannot have both a `query_all` field and `query` fields",
            Self::All(_) => "cannot have multiple `query_all` fields",
        };

        Err(syn::Error::new(Span::call_site(), error_msg))
    }
}

impl RequestBody {
    /// Add the given field to the list of [`RequestBody::JsonFields`].
    ///
    /// Returns an error if this is not a [`RequestBody::Empty`] or [`RequestBody::JsonFields`].
    fn push_json_field(&mut self, field: syn::Field) -> syn::Result<()> {
        let error_msg = match self {
            Self::Empty => {
                *self = Self::JsonFields(vec![field]);
                return Ok(());
            }
            Self::JsonFields(fields) => {
                fields.push(field);
                return Ok(());
            }
            Self::JsonAll(_) => "cannot have both a `body` field and regular body fields",
            Self::Raw(_) => "cannot have both a `raw_body` field and regular body fields",
        };

        Err(syn::Error::new(Span::call_site(), error_msg))
    }

    /// Set this as a [`RequestBody::JsonAll`] with the given field.
    ///
    /// Returns an error if this is not a [`RequestBody::Empty`].
    fn set_json_all(&mut self, field: syn::Field) -> syn::Result<()> {
        let error_msg = match self {
            Self::Empty => {
                *self = Self::JsonAll(field);
                return Ok(());
            }
            Self::JsonFields(_) => "cannot have both a `body` field and regular body fields",
            Self::JsonAll(_) => "cannot have multiple `body` fields",
            Self::Raw(_) => "cannot have both a `raw_body` field and a `body` field",
        };

        Err(syn::Error::new(Span::call_site(), error_msg))
    }

    /// Set this as a [`RequestBody::Raw`] with the given field.
    ///
    /// Returns an error if this is not a [`RequestBody::Empty`].
    fn set_raw(&mut self, field: syn::Field) -> syn::Result<()> {
        let error_msg = match self {
            Self::Empty => {
                *self = Self::Raw(field);
                return Ok(());
            }
            Self::JsonFields(_) => "cannot have both a `raw_body` field and regular body fields",
            Self::JsonAll(_) => "cannot have both a `raw_body` field and a `body` field",
            Self::Raw(_) => "cannot have multiple `raw_body` fields",
        };

        Err(syn::Error::new(Span::call_site(), error_msg))
    }

    /// Validate the fields in this `RequestBody`.
    fn validate(&self) -> syn::Result<()> {
        if let Self::JsonFields(fields) = self
            && fields.len() == 1
            && let Some(single_field) = fields.first()
            && single_field.has_serde_flatten_attribute()
        {
            return Err(syn::Error::new_spanned(
                single_field,
                "Use `#[ruma_api(body)]` to represent the JSON body as a single field",
            ));
        }

        Ok(())
    }
}

/// Attributes on the request struct.
#[derive(Default)]
pub(crate) struct RequestAttrs {
    /// The type used for the `EndpointError` associated type on `OutgoingRequest` and
    /// `IncomingRequest` implementations.
    error_ty: Option<syn::Type>,
}

impl RequestAttrs {
    /// Set the error type of this `RequestAttrs`.
    ///
    /// Returns an error if the error type is already set.
    fn set_error_ty(&mut self, error_ty: syn::Type) -> syn::Result<()> {
        if self.error_ty.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "cannot have multiple values for `error` request attribute",
            ));
        }

        self.error_ty = Some(error_ty);
        Ok(())
    }

    /// Try to parse the given meta item and merge it into this `RequestAttrs`.
    ///
    /// Returns an error if parsing the meta item fails, or if it sets a field that was already set.
    pub(crate) fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.path.is_ident("error") {
            return self.set_error_ty(meta.value()?.parse()?);
        }

        Err(meta.error("unsupported `request` attribute"))
    }

    /// The error type that was set on the request, or the default value which is `MatrixError`.
    pub(super) fn error_ty_or_default(&self, ruma_common: &TokenStream) -> syn::Type {
        self.error_ty
            .clone()
            .unwrap_or_else(|| parse_quote! { #ruma_common::api::error::MatrixError })
    }
}

/// A parsed field of a request struct.
struct RequestField {
    /// The field with the `ruma_api` attributes stripped.
    inner: syn::Field,

    /// The kind of field.
    kind: RequestFieldKind,
}

impl RequestField {
    /// Set the kind of this `RequestField`.
    ///
    /// Returns an error if the kind was already set.
    fn set_kind(&mut self, kind: RequestFieldKind) -> syn::Result<()> {
        if !matches!(self.kind, RequestFieldKind::Body) {
            return Err(syn::Error::new_spanned(
                &self.inner,
                "multiple request field kind attributes found, there can only be one",
            ));
        }

        self.kind = kind;
        Ok(())
    }

    /// Try to merge the values of the attributes in the given meta in this `RequestField`.
    ///
    /// Returns an error if parsing the meta fails or a value is set twice.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if let Some(kind) = RequestFieldKind::try_from_meta(&meta)? {
            return self.set_kind(kind);
        }

        Err(meta.error("unsupported `ruma_api` field attribute"))
    }
}

impl TryFrom<syn::Field> for RequestField {
    type Error = syn::Error;

    fn try_from(inner: syn::Field) -> syn::Result<Self> {
        let mut field = RequestField { inner, kind: Default::default() };

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

/// The kind of a request field.
#[derive(Default)]
enum RequestFieldKind {
    /// Part of the JSON data in the body of the request.
    #[default]
    Body,

    /// The full JSON data in the body of the request.
    NewtypeBody,

    /// Arbitrary bytes in the body of the request.
    RawBody,

    /// Data that appears in the URL path.
    Path,

    /// Data that appears in the query string.
    Query,

    /// Data that represents all the query string as a single type.
    QueryAll,

    /// Data in an HTTP header.
    Header {
        /// The name of the header, as a constant from `http::header`.
        name: syn::Ident,
    },
}

impl RequestFieldKind {
    /// Try to convert the given meta into a `RequestFieldKind`.
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
                if !meta.input.is_empty() {
                    return Err(meta.error("`body` attribute doesn't expect a value"));
                }

                Ok(Some(Self::NewtypeBody))
            }
            "raw_body" => {
                if !meta.input.is_empty() {
                    return Err(meta.error("`raw_body` attribute doesn't expect a value"));
                }

                Ok(Some(Self::RawBody))
            }
            "path" => {
                if !meta.input.is_empty() {
                    return Err(meta.error("`path` attribute doesn't expect a value"));
                }

                Ok(Some(Self::Path))
            }
            "query" => {
                if !meta.input.is_empty() {
                    return Err(meta.error("`query` attribute doesn't expect a value"));
                }

                Ok(Some(Self::Query))
            }
            "query_all" => {
                if !meta.input.is_empty() {
                    return Err(meta.error("`query_all` attribute doesn't expect a value"));
                }

                Ok(Some(Self::QueryAll))
            }
            "header" => {
                let name = meta.value()?.parse()?;
                Ok(Some(Self::Header { name }))
            }
            _ => Ok(None),
        }
    }
}
