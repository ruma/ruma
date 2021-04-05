//! Details of the `ruma_api` procedural macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

pub(crate) mod attribute;
pub(crate) mod metadata;
pub(crate) mod parse;
pub(crate) mod request;
pub(crate) mod response;

use self::{metadata::Metadata, request::Request, response::Response};
use crate::util;

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

pub fn expand_all(api: Api) -> syn::Result<TokenStream> {
    let ruma_api = util::import_ruma_api();
    let http = quote! { #ruma_api::exports::http };

    let metadata = &api.metadata;
    let description = &metadata.description;
    let method = &metadata.method;
    let name = &metadata.name;
    let path = &metadata.path;
    let rate_limited: TokenStream = metadata
        .rate_limited
        .iter()
        .map(|r| {
            let attrs = &r.attrs;
            let value = &r.value;
            quote! {
                #( #attrs )*
                rate_limited: #value,
            }
        })
        .collect();
    let authentication: TokenStream = api
        .metadata
        .authentication
        .iter()
        .map(|r| {
            let attrs = &r.attrs;
            let value = &r.value;
            quote! {
                #( #attrs )*
                authentication: #ruma_api::AuthScheme::#value,
            }
        })
        .collect();

    let error_ty =
        api.error_ty.map_or_else(|| quote! { #ruma_api::error::Void }, |err_ty| quote! { #err_ty });

    let request = api.request.map(|req| req.expand(metadata, &error_ty, &ruma_api));
    let response = api.response.map(|res| res.expand(metadata, &error_ty, &ruma_api));

    let metadata_doc = format!("Metadata for the `{}` API endpoint.", name.value());

    Ok(quote! {
        #[doc = #metadata_doc]
        pub const METADATA: #ruma_api::Metadata = #ruma_api::Metadata {
            description: #description,
            method: #http::Method::#method,
            name: #name,
            path: #path,
            #rate_limited
            #authentication
        };

        #request
        #response
    })
}
