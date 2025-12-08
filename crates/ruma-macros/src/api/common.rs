//! Common types used by the request and response macros.

use std::collections::BTreeMap;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse_quote;

use crate::util::{
    PrivateField, RumaCommon, RumaCommonReexport, SerdeMetaItem, StructFieldExt, TypeExt,
    expand_fields_as_list, expand_fields_as_variable_declarations,
};

/// Parsed HTTP headers of a request or response struct.
#[derive(Default)]
pub(super) struct Headers(BTreeMap<syn::Ident, syn::Field>);

impl Headers {
    /// Insert the given header to this `Headers`.
    ///
    /// Returns an error if the given header is already set.
    pub(super) fn insert(&mut self, header: syn::Ident, field: syn::Field) -> syn::Result<()> {
        if self.0.contains_key(&header) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("cannot have multiple values for `{header}` header"),
            ));
        }

        self.0.insert(header, field);

        Ok(())
    }

    /// Generate code for a comma-separated list of field names.
    ///
    /// Only the `#[cfg]` attributes on the fields are forwarded.
    pub(super) fn expand_fields(&self) -> TokenStream {
        expand_fields_as_list(self.0.values())
    }

    /// Generate code to parse the headers from an `http::request::Request` or
    /// `http::response::Response`.
    pub(super) fn expand_parse(
        &self,
        kind: MacroKind,
        ruma_common: &RumaCommon,
    ) -> Option<TokenStream> {
        if self.0.is_empty() {
            return None;
        }

        let src = kind.as_variable_ident();
        let decls = self
            .0
            .iter()
            .map(|(header_name, field)| Self::expand_parse_header(header_name, field, ruma_common));

        Some(quote! {
            let headers = #src.headers();

            #( #decls )*
        })
    }

    /// Generate code to parse the header with the given name, to assign it to a variable for the
    /// given field, by extracting it from a `http::header::HeaderMap` named `headers`.
    pub(super) fn expand_parse_header(
        header_name: &syn::Ident,
        field: &syn::Field,
        ruma_common: &RumaCommon,
    ) -> TokenStream {
        let ident = field.ident();
        let cfg_attrs = field.cfg_attrs();
        let header_name_string = header_name.to_string();
        let field_type = &field.ty;

        // We need to handle optional fields manually, because we need to parse the inner type.
        let option_inner_type = field_type.option_inner_type();

        let some_case = if let Some(field_type) = option_inner_type {
            quote! {
                str_value.parse::<#field_type>().ok()
            }
        } else {
            quote! {
                str_value
                    .parse::<#field_type>()
                    .map_err(|e| #ruma_common::api::error::HeaderDeserializationError::InvalidHeader(e.into()))?
            }
        };

        let none_case = if option_inner_type.is_some() {
            quote! { None }
        } else {
            quote! {
                return Err(
                    #ruma_common::api::error::HeaderDeserializationError::MissingHeader(
                        #header_name_string.into()
                    ).into(),
                )
            }
        };

        quote! {
            #( #cfg_attrs )*
            let #ident = match headers.get(#header_name) {
                Some(header_value) => {
                    let str_value = header_value.to_str()?;
                    #some_case
                }
                None => #none_case,
            };
        }
    }

    /// Generate code to serialize the headers for a `http::request::Request` or
    /// `http::response::Response`.
    pub(super) fn expand_serialize(
        &self,
        kind: MacroKind,
        body: &Body,
        ruma_common: &RumaCommon,
        http: &TokenStream,
    ) -> Option<TokenStream> {
        let mut serialize = TokenStream::new();

        // If there is no `CONTENT_TYPE` header, add one if necessary.
        let content_type: syn::Ident = parse_quote!(CONTENT_TYPE);
        if !self.0.contains_key(&content_type)
            && let Some(content_type) = body.content_type(kind)
        {
            serialize.extend(quote! {
                headers.insert(
                    #http::header::CONTENT_TYPE,
                    #ruma_common::http_headers::#content_type,
                );
            });
        }

        if serialize.is_empty() && self.0.is_empty() {
            return None;
        }

        for (header_name, field) in &self.0 {
            let ident = field.ident();
            let cfg_attrs = field.cfg_attrs();

            let header = if field.ty.option_inner_type().is_some() {
                quote! {
                    #( #cfg_attrs )*
                    if let Some(header_val) = #ident.as_ref() {
                        headers.insert(
                            #header_name,
                            #http::header::HeaderValue::from_str(&header_val.to_string())?,
                        );
                    }
                }
            } else {
                quote! {
                    #( #cfg_attrs )*
                    headers.insert(
                        #header_name,
                        #http::header::HeaderValue::from_str(&#ident.to_string())?,
                    );
                }
            };

            serialize.extend(header);
        }

        let src = kind.as_variable_ident();

        Some(quote! {{
            let headers = #src.headers_mut();
            #serialize
        }})
    }
}

/// Parsed body of a request or response struct.
#[derive(Default)]
pub(super) struct Body {
    /// The fields containing the data of the body.
    fields: BodyFields,

    /// Whether the body serde type `Serialize` and `Deserialize` implementations are done
    /// manually.
    manual_serde: bool,
}

impl Body {
    /// Add the given field to the list of JSON data fields.
    ///
    /// Returns an error if the fields are not empty or JSON data fields.
    pub(super) fn push_json_field(&mut self, field: syn::Field) -> syn::Result<()> {
        self.fields.push_json_field(field)
    }

    /// Set the given field as the full JSON data.
    ///
    /// Returns an error if the fields are not empty.
    pub(super) fn set_json_all(&mut self, field: syn::Field) -> syn::Result<()> {
        self.fields.set_json_all(field)
    }

    /// Set the given field as the full raw data.
    ///
    /// Returns an error if the fields are not empty.
    pub(super) fn set_raw(&mut self, field: syn::Field) -> syn::Result<()> {
        self.fields.set_raw(field)
    }

    /// Set whether the body serde type `Serialize` and `Deserialize` implementations are done
    /// manually.
    pub(super) fn set_manual_serde(&mut self, manual_serde: bool) {
        self.manual_serde = manual_serde;
    }

    /// Whether the fields are empty.
    pub(super) fn is_empty(&self) -> bool {
        matches!(self.fields, BodyFields::Empty)
    }

    /// Validate the fields in this `Body`.
    pub(super) fn validate(&self) -> syn::Result<()> {
        if let BodyFields::JsonFields(fields) = &self.fields
            && fields.len() == 1
            && let Some(single_field) = fields.first()
            && single_field.has_serde_meta_item(SerdeMetaItem::Flatten)
        {
            return Err(syn::Error::new_spanned(
                single_field,
                "Use `#[ruma_api(body)]` to represent the JSON body as a single field",
            ));
        }

        if matches!(self.fields, BodyFields::Raw(_)) && self.manual_serde {
            return Err(syn::Error::new(
                Span::call_site(),
                "Cannot have a `manual_body_serde` container attribute with a `raw_body` field attribute",
            ));
        }

        Ok(())
    }

    /// The content type of the body, if it can be determined.
    ///
    /// Returns a `const` from `ruma_common::http_headers`.
    fn content_type(&self, kind: MacroKind) -> Option<syn::Ident> {
        match &self.fields {
            BodyFields::Empty if matches!(kind, MacroKind::Request) => {
                // If there are no body fields, the request body might be empty (not `{}`), so the
                // `application/json` content-type would be wrong. It may also cause problems with
                // CORS policies that don't allow the `Content-Type` header (for things such as
                // `.well-known` that are commonly handled by something else than a
                // homeserver). However, a server should always return a JSON body.
                None
            }
            BodyFields::Empty | BodyFields::JsonFields(_) | BodyFields::JsonAll(_) => {
                Some(parse_quote! { APPLICATION_JSON })
            }
            // This might not be the actual content type, but this is a better default than
            // `application/json` when sending raw data.
            BodyFields::Raw(_) => Some(parse_quote! { APPLICATION_OCTET_STREAM }),
        }
    }

    /// Generate code for a comma-separated list of field names.
    ///
    /// Only the `#[cfg]` attributes on the fields are forwarded.
    pub(super) fn expand_fields(&self) -> Option<TokenStream> {
        self.fields.expand_fields()
    }

    /// Generate code to define a `struct {ident_prefix}Body` used for (de)serializing the JSON body
    /// of request or response.
    pub(super) fn expand_serde_struct_definition(
        &self,
        kind: MacroKind,
        ruma_common: &RumaCommon,
    ) -> Option<TokenStream> {
        let fields = self.fields.json_fields()?.iter().map(PrivateField);
        let ident = kind.as_struct_ident(StructSuffix::Body);

        let ruma_macros = ruma_common.reexported(RumaCommonReexport::RumaMacros);
        let mut extra_attrs = TokenStream::new();

        if !self.manual_serde {
            let serde = ruma_common.reexported(RumaCommonReexport::Serde);

            let serialize_feature = match kind {
                MacroKind::Request => "client",
                MacroKind::Response => "server",
            };
            let deserialize_feature = match kind {
                MacroKind::Request => "server",
                MacroKind::Response => "client",
            };

            extra_attrs.extend(quote! {
                #[cfg_attr(feature = #serialize_feature, derive(#serde::Serialize))]
                #[cfg_attr(feature = #deserialize_feature, derive(#serde::Deserialize))]
            });
        }

        if matches!(self.fields, BodyFields::JsonAll(_)) {
            extra_attrs.extend(quote! { #[serde(transparent)] });
        }

        Some(quote! {
            /// Data in the request body.
            #[cfg(any(feature = "client", feature = "server"))]
            #[derive(Debug, #ruma_macros::_FakeDeriveRumaApi, #ruma_macros::_FakeDeriveSerde)]
            #extra_attrs
            struct #ident { #( #fields ),* }
        })
    }

    /// Generate code to parse the body from an `http::request::Request` or
    /// `http::response::Response` named `src`.
    pub(super) fn expand_parse(
        &self,
        kind: MacroKind,
        ruma_common: &RumaCommon,
    ) -> Option<TokenStream> {
        match &self.fields {
            BodyFields::Empty => None,
            BodyFields::JsonFields(fields) => {
                Some(Self::expand_parse_json_body(fields, kind, ruma_common))
            }
            BodyFields::JsonAll(field) => {
                Some(Self::expand_parse_json_body(std::slice::from_ref(field), kind, ruma_common))
            }
            BodyFields::Raw(field) => {
                let src = kind.as_variable_ident();
                let ident = field.ident();
                let cfg_attrs = field.cfg_attrs();

                Some(quote! {
                    #( #cfg_attrs )*
                    let #ident =
                        ::std::convert::AsRef::<[u8]>::as_ref(#src.body()).to_vec();
                })
            }
        }
    }

    /// Generate code to parse a JSON body with the given fields, to assign it to a variable
    /// for the given fields.
    fn expand_parse_json_body(
        fields: &[syn::Field],
        kind: MacroKind,
        ruma_common: &RumaCommon,
    ) -> TokenStream {
        let body = parse_quote! { body };
        let src = kind.as_variable_ident();
        let ident = kind.as_struct_ident(StructSuffix::Body);
        let serde_json = ruma_common.reexported(RumaCommonReexport::SerdeJson);

        let assignments = expand_fields_as_variable_declarations(fields, &body);

        quote! {
            let #body: #ident = {
                let body = ::std::convert::AsRef::<[::std::primitive::u8]>::as_ref(
                    #src.body(),
                );

                #serde_json::from_slice(match body {
                    // If the body is completely empty, pretend it is an empty JSON object instead.
                    // This allows bodies with only optional fields to be deserialized in that case.
                    [] => b"{}",
                    b => b,
                })?
            };

            #assignments
        }
    }

    /// Generate code to serialize the body.
    pub(super) fn expand_serialize(
        &self,
        kind: MacroKind,
        ruma_common: &RumaCommon,
    ) -> TokenStream {
        match &self.fields {
            BodyFields::Empty => match kind {
                MacroKind::Request => {
                    quote! { <Self as #ruma_common::api::Metadata>::empty_request_body::<T>() }
                }
                // A response always returns a JSON body.
                MacroKind::Response => quote! { #ruma_common::serde::slice_to_buf(b"{}") },
            },
            BodyFields::JsonFields(_) => self.expand_serialize_json(kind, ruma_common),
            BodyFields::JsonAll(_) => self.expand_serialize_json(kind, ruma_common),
            BodyFields::Raw(field) => {
                let ident = field.ident();
                quote! { #ruma_common::serde::slice_to_buf(&#ident) }
            }
        }
    }

    /// Generate code to serialize the JSON body with the given fields.
    fn expand_serialize_json(&self, kind: MacroKind, ruma_common: &RumaCommon) -> TokenStream {
        let fields = self.expand_fields();
        let serde_struct = kind.as_struct_ident(StructSuffix::Body);

        quote! {
            #ruma_common::serde::json_to_buf(&#serde_struct { #fields })?
        }
    }
}

/// Parsed body fields.
#[derive(Default)]
enum BodyFields {
    /// The response has an empty body.
    ///
    /// An empty body might contain no data at all or an empty JSON object, depending on the
    /// expected content type of the response.
    #[default]
    Empty,

    /// The body is a JSON object containing the given fields.
    JsonFields(Vec<syn::Field>),

    /// The body is a JSON object represented by the given single field.
    JsonAll(syn::Field),

    /// The body contains raw data represented by the given single field.
    Raw(syn::Field),
}

impl BodyFields {
    /// Add the given field to the list of [`BodyFields::JsonFields`].
    ///
    /// Returns an error if this is not a [`BodyFields::Empty`] or [`BodyFields::JsonFields`].
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

    /// Set this as a [`BodyFields::JsonAll`] with the given field.
    ///
    /// Returns an error if this is not a [`BodyFields::Empty`].
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

    /// Set this as a [`BodyFields::Raw`] with the given field.
    ///
    /// Returns an error if this is not a [`BodyFields::Empty`].
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

    /// The list of fields for the JSON data of the body.
    fn json_fields(&self) -> Option<&[syn::Field]> {
        let fields = match self {
            Self::Empty | Self::Raw(_) => return None,
            Self::JsonFields(fields) => fields.as_slice(),
            Self::JsonAll(field) => std::slice::from_ref(field),
        };

        Some(fields)
    }

    /// Generate code for a comma-separated list of field names.
    ///
    /// Only the `#[cfg]` attributes on the fields are forwarded.
    fn expand_fields(&self) -> Option<TokenStream> {
        let fields = match self {
            Self::Empty => return None,
            Self::JsonFields(fields) => fields.as_slice(),
            Self::JsonAll(field) => std::slice::from_ref(field),
            Self::Raw(field) => std::slice::from_ref(field),
        };

        Some(expand_fields_as_list(fields))
    }
}

/// The kind of macro we are currently implementing.
///
/// This is used to generate variables and structs names.
#[derive(Clone, Copy)]
pub(super) enum MacroKind {
    /// The `request` macro.
    Request,

    /// The `response` macro.
    Response,
}

impl MacroKind {
    /// Generate the name of a variable.
    pub(super) fn as_variable_ident(&self) -> syn::Ident {
        match self {
            Self::Request => parse_quote! { request },
            Self::Response => parse_quote! { response },
        }
    }

    /// Generate the name of a struct with the given suffix.
    pub(super) fn as_struct_ident(&self, suffix: StructSuffix) -> syn::Ident {
        let prefix = match self {
            Self::Request => "Request",
            Self::Response => "Response",
        };

        format_ident!("{prefix}{}", suffix.as_str())
    }
}

/// The supported suffixes for generated structs.
pub(super) enum StructSuffix {
    /// `Body`.
    Body,

    /// `Query`.
    Query,
}

impl StructSuffix {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Body => "Body",
            Self::Query => "Query",
        }
    }
}
