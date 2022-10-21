//! Methods and types for generating API endpoints.

use std::{env, fs, path::Path};

use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::{de::IgnoredAny, Deserialize};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Field, Token, Type,
};

use self::{
    api_metadata::Metadata,
    api_request::Request,
    api_response::Response,
    request::{RequestField, RequestFieldKind},
};
use crate::util::import_ruma_common;

mod api_metadata;
mod api_request;
mod api_response;
mod attribute;
mod auth_scheme;
pub mod request;
pub mod response;
mod util;
mod version;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(error);
    custom_keyword!(request);
    custom_keyword!(response);
}

/// The result of processing the `ruma_api` macro, ready for output back to source code.
pub struct Api {
    /// The `metadata` section of the macro.
    metadata: Metadata,

    /// The `request` section of the macro.
    request: Option<Request>,

    /// The `response` section of the macro.
    response: Option<Response>,

    /// The `error` section of the macro.
    error_ty: Option<Type>,
}

impl Api {
    pub fn expand_all(self) -> TokenStream {
        let maybe_feature_error = ensure_feature_presence().map(syn::Error::to_compile_error);
        let maybe_path_error = self.check_paths().err().map(syn::Error::into_compile_error);

        let ruma_common = import_ruma_common();
        let http = quote! { #ruma_common::exports::http };

        let metadata = &self.metadata;
        let description = &metadata.description;
        let method = &metadata.method;
        let name = &metadata.name;
        let rate_limited = &self.metadata.rate_limited;
        let authentication = &self.metadata.authentication;
        let history = &self.metadata.history;

        let error_ty = self.error_ty.map_or_else(
            || quote! { #ruma_common::api::error::MatrixError },
            |err_ty| quote! { #err_ty },
        );

        let request = self.request.map(|req| req.expand(metadata, &ruma_common));
        let response = self.response.map(|res| res.expand(metadata, &ruma_common));

        let metadata_doc = format!("Metadata for the `{}` API endpoint.", name.value());

        quote! {
            #maybe_feature_error
            #maybe_path_error

            #[doc = #metadata_doc]
            pub const METADATA: #ruma_common::api::Metadata = #ruma_common::api::Metadata {
                description: #description,
                method: #http::Method::#method,
                name: #name,
                rate_limited: #rate_limited,
                authentication: #ruma_common::api::AuthScheme::#authentication,
                history: #history,
            };

            #[allow(unused)]
            type EndpointError = #error_ty;

            #request
            #response
        }
    }

    fn check_paths(&self) -> syn::Result<()> {
        let mut path_iter = self.metadata.history.entries.iter().filter_map(|entry| entry.path());

        let path = path_iter.next().ok_or_else(|| {
            syn::Error::new(Span::call_site(), "at least one path metadata field must be set")
        })?;
        let path_args = path.args();

        for extra_path in path_iter {
            let extra_path_args = extra_path.args();
            if extra_path_args != path_args {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "paths have different path parameters",
                ));
            }
        }

        if let Some(req) = &self.request {
            let path_field_names: Vec<_> = req
                .fields
                .iter()
                .cloned()
                .filter_map(|f| match RequestField::try_from(f) {
                    Ok(RequestField { kind: RequestFieldKind::Path, inner }) => {
                        Some(Ok(inner.ident.unwrap().to_string()))
                    }
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<syn::Result<_>>()?;

            if path_args != path_field_names {
                return Err(syn::Error::new_spanned(
                    req.request_kw,
                    "path fields must be in the same order as they appear in the path segments",
                ));
            }
        }

        Ok(())
    }
}

impl Parse for Api {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata: Metadata = input.parse()?;

        let req_attrs = input.call(Attribute::parse_outer)?;
        let (request, attributes) = if input.peek(kw::request) {
            let request = parse_request(input, req_attrs)?;
            let after_req_attrs = input.call(Attribute::parse_outer)?;

            (Some(request), after_req_attrs)
        } else {
            // There was no `request` field so the attributes are for `response`
            (None, req_attrs)
        };

        let response = if input.peek(kw::response) {
            Some(parse_response(input, attributes)?)
        } else if !attributes.is_empty() {
            return Err(syn::Error::new_spanned(
                &attributes[0],
                "attributes are not supported on the error type",
            ));
        } else {
            None
        };

        let error_ty = input
            .peek(kw::error)
            .then(|| {
                let _: kw::error = input.parse()?;
                let _: Token![:] = input.parse()?;

                input.parse()
            })
            .transpose()?;

        Ok(Self { metadata, request, response, error_ty })
    }
}

fn parse_request(input: ParseStream<'_>, attributes: Vec<Attribute>) -> syn::Result<Request> {
    let request_kw: kw::request = input.parse()?;
    let _: Token![:] = input.parse()?;
    let fields;
    braced!(fields in input);

    let fields = fields.parse_terminated::<_, Token![,]>(Field::parse_named)?;

    Ok(Request { request_kw, attributes, fields })
}

fn parse_response(input: ParseStream<'_>, attributes: Vec<Attribute>) -> syn::Result<Response> {
    let response_kw: kw::response = input.parse()?;
    let _: Token![:] = input.parse()?;
    let fields;
    braced!(fields in input);

    let fields = fields.parse_terminated::<_, Token![,]>(Field::parse_named)?;

    Ok(Response { attributes, fields, response_kw })
}

// Returns an error with a helpful error if the crate `ruma_api!` is used from doesn't declare both
// a `client` and a `server` feature.
fn ensure_feature_presence() -> Option<&'static syn::Error> {
    #[derive(Deserialize)]
    struct CargoToml {
        features: Features,
    }

    #[derive(Deserialize)]
    struct Features {
        client: Option<IgnoredAny>,
        server: Option<IgnoredAny>,
    }

    static RESULT: Lazy<Result<(), syn::Error>> = Lazy::new(|| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| syn::Error::new(Span::call_site(), "Failed to read CARGO_MANIFEST_DIR"))?;

        let manifest_file = Path::new(&manifest_dir).join("Cargo.toml");
        let manifest_bytes = fs::read(manifest_file)
            .map_err(|_| syn::Error::new(Span::call_site(), "Failed to read Cargo.toml"))?;

        let manifest_parsed: CargoToml = toml::from_slice(&manifest_bytes)
            .map_err(|_| syn::Error::new(Span::call_site(), "Failed to parse Cargo.toml"))?;

        if manifest_parsed.features.client.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "This crate doesn't define a `client` feature in its `Cargo.toml`.\n\
                 Please add a `client` feature such that generated `OutgoingRequest` and \
                 `IncomingResponse` implementations can be enabled.",
            ));
        }

        if manifest_parsed.features.server.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "This crate doesn't define a `server` feature in its `Cargo.toml`.\n\
                 Please add a `server` feature such that generated `IncomingRequest` and \
                 `OutgoingResponse` implementations can be enabled.",
            ));
        }

        Ok(())
    });

    RESULT.as_ref().err()
}
