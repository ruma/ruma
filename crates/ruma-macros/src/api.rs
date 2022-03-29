//! Methods and types for generating API endpoints.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Field, Token, Type,
};

use self::{api_metadata::Metadata, api_request::Request, api_response::Response};
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
        let ruma_common = import_ruma_common();
        let http = quote! { #ruma_common::exports::http };

        let metadata = &self.metadata;
        let description = &metadata.description;
        let method = &metadata.method;
        let name = &metadata.name;
        let unstable_path = util::map_option_literal(&metadata.unstable_path);
        let r0_path = util::map_option_literal(&metadata.r0_path);
        let stable_path = util::map_option_literal(&metadata.stable_path);
        let rate_limited = &self.metadata.rate_limited;
        let authentication = &self.metadata.authentication;
        let added = util::map_option_literal(&metadata.added);
        let deprecated = util::map_option_literal(&metadata.deprecated);
        let removed = util::map_option_literal(&metadata.removed);

        let error_ty = self.error_ty.map_or_else(
            || quote! { #ruma_common::api::error::MatrixError },
            |err_ty| quote! { #err_ty },
        );

        let request = self.request.map(|req| req.expand(metadata, &error_ty, &ruma_common));
        let response = self.response.map(|res| res.expand(metadata, &error_ty, &ruma_common));

        let metadata_doc = format!("Metadata for the `{}` API endpoint.", name.value());

        quote! {
            #[doc = #metadata_doc]
            pub const METADATA: #ruma_common::api::Metadata = #ruma_common::api::Metadata {
                description: #description,
                method: #http::Method::#method,
                name: #name,
                unstable_path: #unstable_path,
                r0_path: #r0_path,
                stable_path: #stable_path,
                added: #added,
                deprecated: #deprecated,
                removed: #removed,
                rate_limited: #rate_limited,
                authentication: #ruma_common::api::AuthScheme::#authentication,
            };

            #request
            #response

            #[cfg(not(any(feature = "client", feature = "server")))]
            type _SilenceUnusedError = #error_ty;
        }
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
