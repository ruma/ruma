//! Details of the `ruma_api` procedural macro.

use std::convert::{TryFrom, TryInto as _};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Field, FieldValue, Ident, Token,
};

mod attribute;
mod metadata;
mod request;
mod response;

use self::{metadata::Metadata, request::Request, response::Response};

/// Removes `serde` attributes from struct fields.
pub fn strip_serde_attrs(field: &Field) -> Field {
    let mut field = field.clone();
    field
        .attrs
        .retain(|attr| attr.path.segments.len() != 1 || attr.path.segments[0].ident != "serde");
    field
}

/// The result of processing the `ruma_api` macro, ready for output back to source code.
pub struct Api {
    /// The `metadata` section of the macro.
    metadata: Metadata,
    /// The `request` section of the macro.
    request: Request,
    /// The `response` section of the macro.
    response: Response,
}

impl TryFrom<RawApi> for Api {
    type Error = syn::Error;

    fn try_from(raw_api: RawApi) -> syn::Result<Self> {
        let res = Self {
            metadata: raw_api.metadata.try_into()?,
            request: raw_api.request.try_into()?,
            response: raw_api.response.try_into()?,
        };

        assert!(
            !(res.metadata.method == "GET"
                && (res.request.has_body_fields() || res.request.newtype_body_field().is_some())),
            "GET endpoints can't have body fields"
        );

        Ok(res)
    }
}

impl ToTokens for Api {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = &self.metadata.description;
        let method = &self.metadata.method;
        // We don't (currently) use this literal as a literal in the generated code. Instead we just
        // put it into doc comments, for which the span information is irrelevant. So we can work
        // with only the literal's value from here on.
        let name = &self.metadata.name.value();
        let path = &self.metadata.path;
        let rate_limited = &self.metadata.rate_limited;
        let requires_authentication = &self.metadata.requires_authentication;

        let request = &self.request;
        let request_types = quote! { #request };
        let response = &self.response;
        let response_types = quote! { #response };

        let set_request_path = if self.request.has_path_fields() {
            let path_str = path.value();

            assert!(path_str.starts_with('/'), "path needs to start with '/'");
            assert!(
                path_str.chars().filter(|c| *c == ':').count() == self.request.path_field_count(),
                "number of declared path parameters needs to match amount of placeholders in path"
            );

            let request_path_init_fields = self.request.request_path_init_fields();

            let path_segments = path_str[1..].split('/');
            let path_segment_push = path_segments.map(|segment| {
                let arg = if segment.starts_with(':') {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::new(path_var, Span::call_site());
                    quote!(&request_path.#path_var_ident.to_string())
                } else {
                    quote!(#segment)
                };

                quote! {
                    path_segments.push(#arg);
                }
            });

            quote! {
                let request_path = RequestPath {
                    #request_path_init_fields
                };

                // This `unwrap()` can only fail when the url is a
                // cannot-be-base url like `mailto:` or `data:`, which is not
                // the case for our placeholder url.
                let mut path_segments = url.path_segments_mut().unwrap();
                #(#path_segment_push)*
            }
        } else {
            quote! {
                url.set_path(metadata.path);
            }
        };

        let set_request_query = if self.request.has_query_fields() {
            let request_query_init_fields = self.request.request_query_init_fields();

            quote! {
                let request_query = RequestQuery {
                    #request_query_init_fields
                };

                url.set_query(Some(&ruma_api::exports::serde_urlencoded::to_string(
                    request_query,
                )?));
            }
        } else {
            TokenStream::new()
        };

        let add_headers_to_request = if self.request.has_header_fields() {
            let add_headers = self.request.add_headers_to_request();
            quote! {
                let headers = http_request.headers_mut();
                #add_headers
            }
        } else {
            TokenStream::new()
        };

        let create_http_request = if let Some(field) = self.request.newtype_body_field() {
            let field_name = field
                .ident
                .as_ref()
                .expect("expected field to have an identifier");

            quote! {
                let request_body = RequestBody(request.#field_name);

                let mut http_request = ruma_api::exports::http::Request::new(
                    ruma_api::exports::serde_json::to_vec(&request_body)?,
                );
            }
        } else if self.request.has_body_fields() {
            let request_body_init_fields = self.request.request_body_init_fields();

            quote! {
                let request_body = RequestBody {
                    #request_body_init_fields
                };

                let mut http_request = ruma_api::exports::http::Request::new(
                    ruma_api::exports::serde_json::to_vec(&request_body)?,
                );
            }
        } else {
            quote! {
                let mut http_request = ruma_api::exports::http::Request::new(Vec::new());
            }
        };

        let try_deserialize_response_body = if let Some(field) = self.response.newtype_body_field()
        {
            let field_type = &field.ty;

            quote! {
                ruma_api::exports::serde_json::from_slice::<#field_type>(
                    http_response.into_body().as_slice(),
                )?
            }
        } else if self.response.has_body_fields() {
            quote! {
                ruma_api::exports::serde_json::from_slice::<ResponseBody>(
                    http_response.into_body().as_slice(),
                )?
            }
        } else {
            quote! {
                ()
            }
        };

        let extract_response_headers = if self.response.has_header_fields() {
            quote! {
                let mut headers = http_response.headers().clone();
            }
        } else {
            TokenStream::new()
        };

        let response_init_fields = if self.response.has_fields() {
            self.response.init_fields()
        } else {
            TokenStream::new()
        };

        let request_doc = format!(
            "Data for a request to the `{}` API endpoint.\n\n{}",
            name,
            description.value()
        );
        let response_doc = format!("Data in the response from the `{}` API endpoint.", name);

        let api = quote! {
            use ruma_api::exports::serde::de::{Error as _, IntoDeserializer as _};
            use ruma_api::exports::serde::Deserialize as _;
            use ruma_api::Endpoint as _;

            use std::convert::TryInto as _;

            #[doc = #request_doc]
            #request_types

            impl std::convert::TryFrom<Request> for ruma_api::exports::http::Request<Vec<u8>> {
                type Error = ruma_api::Error;

                #[allow(unused_mut, unused_variables)]
                fn try_from(request: Request) -> Result<Self, Self::Error> {
                    let metadata = Request::METADATA;

                    // Use dummy homeserver url which has to be overwritten in
                    // the calling code. Previously (with http::Uri) this was
                    // not required, but Url::parse only accepts absolute urls.
                    let mut url =
                        ruma_api::exports::url::Url::parse("http://invalid-host-please-change/")
                            .unwrap();

                    { #set_request_path }
                    { #set_request_query }

                    #create_http_request

                    *http_request.method_mut() = ruma_api::exports::http::Method::#method;
                    *http_request.uri_mut() = url.into_string().parse().unwrap();

                    { #add_headers_to_request }

                    Ok(http_request)
                }
            }

            #[doc = #response_doc]
            #response_types

            impl std::convert::TryFrom<ruma_api::exports::http::Response<Vec<u8>>> for Response {
                type Error = ruma_api::Error;

                #[allow(unused_variables)]
                fn try_from(
                    http_response: ruma_api::exports::http::Response<Vec<u8>>,
                ) -> Result<Self, Self::Error> {
                    if http_response.status().is_success() {
                        #extract_response_headers

                        let response_body = #try_deserialize_response_body;
                        Ok(Response {
                            #response_init_fields
                        })
                    } else {
                        Err(http_response.status().clone().into())
                    }
                }
            }

            impl ruma_api::Endpoint for Request {
                type Response = Response;

                /// Metadata for the `#name` endpoint.
                const METADATA: ruma_api::Metadata = ruma_api::Metadata {
                    description: #description,
                    method: ruma_api::exports::http::Method::#method,
                    name: #name,
                    path: #path,
                    rate_limited: #rate_limited,
                    requires_authentication: #requires_authentication,
                };
            }
        };

        api.to_tokens(tokens);
    }
}

/// Custom keyword macros for syn.
mod kw {
    use syn::custom_keyword;

    custom_keyword!(metadata);
    custom_keyword!(request);
    custom_keyword!(response);
}

/// The entire `ruma_api!` macro structure directly as it appears in the source code..
pub struct RawApi {
    /// The `metadata` section of the macro.
    pub metadata: RawMetadata,
    /// The `request` section of the macro.
    pub request: RawRequest,
    /// The `response` section of the macro.
    pub response: Vec<Field>,
}

impl Parse for RawApi {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata = input.parse::<RawMetadata>()?;
        let request = input.parse::<RawRequest>()?;

        input.parse::<kw::response>()?;
        let response;
        braced!(response in input);

        Ok(Self {
            metadata,
            request,
            response: response
                .parse_terminated::<Field, Token![,]>(Field::parse_named)?
                .into_iter()
                .collect(),
        })
    }
}

pub struct RawMetadata {
    pub metadata_kw: kw::metadata,
    pub field_values: Vec<FieldValue>,
}

impl Parse for RawMetadata {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata_kw = input.parse::<kw::metadata>()?;
        let field_values;
        braced!(field_values in input);

        Ok(Self {
            metadata_kw,
            field_values: field_values
                .parse_terminated::<FieldValue, Token![,]>(FieldValue::parse)?
                .into_iter()
                .collect(),
        })
    }
}

pub struct RawRequest {
    pub request_kw: kw::request,
    pub fields: Vec<Field>,
}

impl Parse for RawRequest {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let request_kw = input.parse::<kw::request>()?;
        let fields;
        braced!(fields in input);

        Ok(Self {
            request_kw,
            fields: fields
                .parse_terminated::<Field, Token![,]>(Field::parse_named)?
                .into_iter()
                .collect(),
        })
    }
}
