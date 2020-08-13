//! Details of the `ruma_api` procedural macro.

use std::convert::{TryFrom, TryInto as _};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Field, FieldValue, Token, Type,
};

pub(crate) mod attribute;
pub(crate) mod metadata;
pub(crate) mod request;
pub(crate) mod response;

use self::{metadata::Metadata, request::Request, response::Response};
use crate::util;

/// Removes `serde` attributes from struct fields.
pub fn strip_serde_attrs(field: &Field) -> Field {
    let mut field = field.clone();
    field.attrs.retain(|attr| !attr.path.is_ident("serde"));
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
    /// The `error` section of the macro.
    error: TokenStream,
}

impl TryFrom<RawApi> for Api {
    type Error = syn::Error;

    fn try_from(raw_api: RawApi) -> syn::Result<Self> {
        let import_path = util::import_ruma_api();

        let res = Self {
            metadata: raw_api.metadata.try_into()?,
            request: raw_api.request.try_into()?,
            response: raw_api.response.try_into()?,
            error: match raw_api.error {
                Some(raw_err) => raw_err.ty.to_token_stream(),
                None => quote! { #import_path::error::Void },
            },
        };

        let newtype_body_field = res.request.newtype_body_field();
        if res.metadata.method == "GET"
            && (res.request.has_body_fields() || newtype_body_field.is_some())
        {
            let mut combined_error: Option<syn::Error> = None;
            let mut add_error = |field| {
                let error = syn::Error::new_spanned(field, "GET endpoints can't have body fields");
                if let Some(combined_error_ref) = &mut combined_error {
                    combined_error_ref.combine(error);
                } else {
                    combined_error = Some(error);
                }
            };

            for field in res.request.body_fields() {
                add_error(field);
            }

            if let Some(field) = newtype_body_field {
                add_error(field);
            }

            Err(combined_error.unwrap())
        } else {
            Ok(res)
        }
    }
}

impl ToTokens for Api {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Guarantee `ruma_api` is available and named something we can refer to.
        let ruma_api_import = util::import_ruma_api();

        let description = &self.metadata.description;
        let method = &self.metadata.method;
        // We don't (currently) use this literal as a literal in the generated code. Instead we just
        // put it into doc comments, for which the span information is irrelevant. So we can work
        // with only the literal's value from here on.
        let name = &self.metadata.name.value();
        let path = &self.metadata.path;
        let rate_limited = &self.metadata.rate_limited;
        let requires_authentication = &self.metadata.requires_authentication;

        let request_type = &self.request;
        let response_type = &self.response;

        let incoming_request_type = if self.request.contains_lifetimes() {
            quote!(IncomingRequest)
        } else {
            quote!(Request)
        };

        let extract_request_path = if self.request.has_path_fields() {
            quote! {
                let path_segments: ::std::vec::Vec<&::std::primitive::str> =
                    request.uri().path()[1..].split('/').collect();
            }
        } else {
            TokenStream::new()
        };

        let (request_path_string, parse_request_path) =
            util::request_path_string_and_parse(&self.request, &self.metadata, &ruma_api_import);

        let request_query_string = util::build_query_string(&self.request, &ruma_api_import);

        let extract_request_query = util::extract_request_query(&self.request, &ruma_api_import);

        let parse_request_query = if let Some(field) = self.request.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");

            quote! {
                #field_name: request_query,
            }
        } else {
            self.request.request_init_query_fields()
        };

        let mut header_kvs = self.request.append_header_kvs();
        if requires_authentication.value {
            header_kvs.push(quote! {
                #ruma_api_import::exports::http::header::AUTHORIZATION,
                #ruma_api_import::exports::http::header::HeaderValue::from_str(
                    &::std::format!(
                        "Bearer {}",
                        access_token.ok_or(
                            #ruma_api_import::error::IntoHttpError::NeedsAuthentication
                        )?
                    )
                )?
            });
        }

        let extract_request_headers = if self.request.has_header_fields() {
            quote! {
                let headers = request.headers();
            }
        } else {
            TokenStream::new()
        };

        let extract_request_body = if self.request.has_body_fields()
            || self.request.newtype_body_field().is_some()
        {
            let body_lifetimes = if self.request.has_body_lifetimes() {
                // duplicate the anonymous lifetime as many times as needed
                let lifetimes =
                    std::iter::repeat(quote! { '_ }).take(self.request.body_lifetime_count());
                quote! { < #( #lifetimes, )* >}
            } else {
                TokenStream::new()
            };
            quote! {
                let request_body: <RequestBody #body_lifetimes as #ruma_api_import::Outgoing>::Incoming =
                    #ruma_api_import::try_deserialize!(
                        request,
                        #ruma_api_import::exports::serde_json::from_slice(request.body().as_slice())
                    );
            }
        } else {
            TokenStream::new()
        };

        let parse_request_headers = if self.request.has_header_fields() {
            self.request.parse_headers_from_request()
        } else {
            TokenStream::new()
        };

        let request_body = util::build_request_body(&self.request, &ruma_api_import);

        let parse_request_body = util::parse_request_body(&self.request);

        let extract_response_headers = if self.response.has_header_fields() {
            quote! {
                let mut headers = response.headers().clone();
            }
        } else {
            TokenStream::new()
        };

        let typed_response_body_decl = if self.response.has_body_fields()
            || self.response.newtype_body_field().is_some()
        {
            quote! {
                let response_body: <ResponseBody as #ruma_api_import::Outgoing>::Incoming =
                    #ruma_api_import::try_deserialize!(
                        response,
                        #ruma_api_import::exports::serde_json::from_slice(response.body().as_slice()),
                    );
            }
        } else {
            TokenStream::new()
        };

        let response_init_fields = self.response.init_fields();

        let serialize_response_headers = self.response.apply_header_fields();

        let body = self.response.to_body();

        let request_doc = format!(
            "Data for a request to the `{}` API endpoint.\n\n{}",
            name,
            description.value()
        );
        let response_doc = format!("Data in the response from the `{}` API endpoint.", name);

        let error = &self.error;

        let request_lifetimes = self.request.combine_lifetimes();

        let non_auth_endpoint_impls = if requires_authentication.value {
            TokenStream::new()
        } else {
            quote! {
                impl #request_lifetimes #ruma_api_import::OutgoingNonAuthRequest
                    for Request #request_lifetimes
                {}

                impl #ruma_api_import::IncomingNonAuthRequest for #incoming_request_type {}
            }
        };

        let api = quote! {
            #[doc = #request_doc]
            #request_type

            impl ::std::convert::TryFrom<#ruma_api_import::exports::http::Request<Vec<u8>>>
                for #incoming_request_type
            {
                type Error = #ruma_api_import::error::FromHttpRequestError;

                #[allow(unused_variables)]
                fn try_from(
                    request: #ruma_api_import::exports::http::Request<Vec<u8>>
                ) -> ::std::result::Result<Self, Self::Error> {
                    #extract_request_path
                    #extract_request_query
                    #extract_request_headers
                    #extract_request_body

                    Ok(Self {
                        #parse_request_path
                        #parse_request_query
                        #parse_request_headers
                        #parse_request_body
                    })
                }
            }

            #[doc = #response_doc]
            #response_type

            impl ::std::convert::TryFrom<Response> for #ruma_api_import::exports::http::Response<Vec<u8>> {
                type Error = #ruma_api_import::error::IntoHttpError;

                #[allow(unused_variables)]
                fn try_from(response: Response) -> ::std::result::Result<Self, Self::Error> {
                    let response = #ruma_api_import::exports::http::Response::builder()
                        .header(#ruma_api_import::exports::http::header::CONTENT_TYPE, "application/json")
                        #serialize_response_headers
                        .body(#body)
                        // Since we require header names to come from the `http::header` module,
                        // this cannot fail.
                        .unwrap();
                    Ok(response)
                }
            }

            impl ::std::convert::TryFrom<#ruma_api_import::exports::http::Response<Vec<u8>>> for Response {
                type Error = #ruma_api_import::error::FromHttpResponseError<#error>;

                #[allow(unused_variables)]
                fn try_from(
                    response: #ruma_api_import::exports::http::Response<Vec<u8>>,
                ) -> ::std::result::Result<Self, Self::Error> {
                    if response.status().as_u16() < 400 {
                        #extract_response_headers

                        #typed_response_body_decl

                        Ok(Self {
                            #response_init_fields
                        })
                    } else {
                        match <#error as #ruma_api_import::EndpointError>::try_from_response(response) {
                            Ok(err) => Err(#ruma_api_import::error::ServerError::Known(err).into()),
                            Err(response_err) => {
                                Err(#ruma_api_import::error::ServerError::Unknown(response_err).into())
                            }
                        }
                    }
                }
            }

            const __METADATA: #ruma_api_import::Metadata = #ruma_api_import::Metadata {
                description: #description,
                method: #ruma_api_import::exports::http::Method::#method,
                name: #name,
                path: #path,
                rate_limited: #rate_limited,
                requires_authentication: #requires_authentication,
            };

            impl #request_lifetimes #ruma_api_import::OutgoingRequest
                for Request #request_lifetimes
            {
                type EndpointError = #error;
                type IncomingResponse = <Response as #ruma_api_import::Outgoing>::Incoming;

                // FIXME: Doc string interpolation
                /// Metadata for the `#name` endpoint.
                const METADATA: #ruma_api_import::Metadata = __METADATA;

                #[allow(unused_mut, unused_variables)]
                fn try_into_http_request(
                    self,
                    base_url: &::std::primitive::str,
                    access_token: ::std::option::Option<&str>,
                ) -> ::std::result::Result<
                    #ruma_api_import::exports::http::Request<Vec<u8>>,
                    #ruma_api_import::error::IntoHttpError,
                > {
                    let metadata = <Self as #ruma_api_import::OutgoingRequest>::METADATA;

                    let http_request = #ruma_api_import::exports::http::Request::builder()
                        .method(#ruma_api_import::exports::http::Method::#method)
                        .uri(::std::format!(
                            "{}{}{}",
                            base_url.strip_suffix("/").unwrap_or(base_url),
                            #request_path_string,
                            #request_query_string,
                        ))
                        #( .header(#header_kvs) )*
                        .body(#request_body)?;

                    Ok(http_request)
                }
            }

            impl #ruma_api_import::IncomingRequest for #incoming_request_type {
                type EndpointError = #error;
                type OutgoingResponse = Response;

                // FIXME: Doc string interpolation
                /// Metadata for the `#name` endpoint.
                const METADATA: #ruma_api_import::Metadata = __METADATA;
            }

            #non_auth_endpoint_impls
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
    custom_keyword!(error);
}

/// The entire `ruma_api!` macro structure directly as it appears in the source code..
pub struct RawApi {
    /// The `metadata` section of the macro.
    pub metadata: RawMetadata,
    /// The `request` section of the macro.
    pub request: RawRequest,
    /// The `response` section of the macro.
    pub response: RawResponse,
    /// The `error` section of the macro.
    pub error: Option<RawErrorType>,
}

impl Parse for RawApi {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            metadata: input.parse()?,
            request: input.parse()?,
            response: input.parse()?,
            error: input.parse().ok(),
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
        input.parse::<Token![:]>()?;
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
    pub attributes: Vec<Attribute>,
    pub request_kw: kw::request,
    pub fields: Vec<Field>,
}

impl Parse for RawRequest {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // If there are attributes to parse otherwise instead of failing
        // emit empty vec.
        let attributes =
            if let Ok(attrs) = input.call(Attribute::parse_outer) { attrs } else { vec![] };

        let request_kw = input.parse::<kw::request>()?;
        input.parse::<Token![:]>()?;
        let fields;
        braced!(fields in input);

        Ok(Self {
            attributes,
            request_kw,
            fields: fields
                .parse_terminated::<Field, Token![,]>(Field::parse_named)?
                .into_iter()
                .collect(),
        })
    }
}

pub struct RawResponse {
    pub attributes: Vec<Attribute>,
    pub response_kw: kw::response,
    pub fields: Vec<Field>,
}

impl Parse for RawResponse {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attributes =
            if let Ok(attrs) = input.call(Attribute::parse_outer) { attrs } else { vec![] };

        let response_kw = input.parse::<kw::response>()?;
        input.parse::<Token![:]>()?;
        let fields;
        braced!(fields in input);

        Ok(Self {
            attributes,
            response_kw,
            fields: fields
                .parse_terminated::<Field, Token![,]>(Field::parse_named)?
                .into_iter()
                .map(|f| {
                    if util::has_lifetime(&f.ty) {
                        Err(syn::Error::new(
                            f.ident.span(),
                            "Lifetimes on Response fields cannot be supported until GAT are stable",
                        ))
                    } else {
                        Ok(f)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub struct RawErrorType {
    pub error_kw: kw::error,
    pub ty: Type,
}

impl Parse for RawErrorType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let error_kw = input.parse::<kw::error>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;

        Ok(Self { error_kw, ty })
    }
}
