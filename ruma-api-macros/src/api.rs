//! Details of the `ruma_api` procedural macro.

use std::convert::{TryFrom, TryInto as _};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Field, FieldValue, Ident, Token, Type,
};

mod attribute;
mod metadata;
mod request;
mod response;

use self::{metadata::Metadata, request::Request, response::Response};

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
    error: Type,
}

impl TryFrom<RawApi> for Api {
    type Error = syn::Error;

    fn try_from(raw_api: RawApi) -> syn::Result<Self> {
        let res = Self {
            metadata: raw_api.metadata.try_into()?,
            request: raw_api.request.try_into()?,
            response: raw_api.response.try_into()?,
            error: raw_api
                .error
                .map_or(syn::parse_str::<Type>("ruma_api::error::Void").unwrap(), |err| err.ty),
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

        let request_try_from_type = if self.request.uses_wrap_incoming() {
            quote!(IncomingRequest)
        } else {
            quote!(Request)
        };

        let response_try_from_type = if self.response.uses_wrap_incoming() {
            quote!(IncomingResponse)
        } else {
            quote!(Response)
        };

        let extract_request_path = if self.request.has_path_fields() {
            quote! {
                let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();
            }
        } else {
            TokenStream::new()
        };

        let (url_set_path, parse_request_path) = if self.request.has_path_fields() {
            let path_str = path.value();

            assert!(path_str.starts_with('/'), "path needs to start with '/'");
            assert!(
                path_str.chars().filter(|c| *c == ':').count() == self.request.path_field_count(),
                "number of declared path parameters needs to match amount of placeholders in path"
            );

            let path_segments = path_str[1..].split('/');
            let path_segment_push = path_segments.clone().map(|segment| {
                let arg = if segment.starts_with(':') {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::new(path_var, Span::call_site());
                    quote!(&request.#path_var_ident.to_string())
                } else {
                    quote!(#segment)
                };

                quote! {
                    path_segments.push(#arg);
                }
            });

            let set_tokens = quote! {
                // This `unwrap()` can only fail when the url is a
                // cannot-be-base url like `mailto:` or `data:`, which is not
                // the case for our placeholder url.
                let mut path_segments = url.path_segments_mut().unwrap();
                #(#path_segment_push)*
            };

            let path_fields = path_segments.enumerate().filter(|(_, s)| s.starts_with(':')).map(
                |(i, segment)| {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::new(path_var, Span::call_site());

                    quote! {
                        #path_var_ident: {
                            use std::ops::Deref as _;
                            use ruma_api::error::RequestDeserializationError;

                            let segment = path_segments.get(#i).unwrap().as_bytes();
                            let decoded = match ruma_api::exports::percent_encoding::percent_decode(
                                segment
                            ).decode_utf8() {
                                Ok(x) => x,
                                Err(err) => {
                                    return Err(
                                        RequestDeserializationError::new(err, request).into()
                                    );
                                }
                            };
                            match std::convert::TryFrom::try_from(decoded.deref()) {
                                Ok(val) => val,
                                Err(err) => {
                                    return Err(
                                        RequestDeserializationError::new(err, request).into()
                                    );
                                }
                            }
                        }
                    }
                },
            );

            let parse_tokens = quote! {
                #(#path_fields,)*
            };

            (set_tokens, parse_tokens)
        } else {
            let set_tokens = quote! {
                url.set_path(metadata.path);
            };
            let parse_tokens = TokenStream::new();
            (set_tokens, parse_tokens)
        };

        let url_set_querystring = if let Some(field) = self.request.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have identifier");
            let field_type = &field.ty;

            quote! {
                // This function exists so that the compiler will throw an
                // error when the type of the field with the query_map
                // attribute doesn't implement IntoIterator<Item = (String, String)>
                //
                // This is necessary because the serde_urlencoded::to_string
                // call will result in a runtime error when the type cannot be
                // encoded as a list key-value pairs (?key1=value1&key2=value2)
                //
                // By asserting that it implements the iterator trait, we can
                // ensure that it won't fail.
                fn assert_trait_impl<T>()
                where
                    T: std::iter::IntoIterator<Item = (std::string::String, std::string::String)>,
                {}
                assert_trait_impl::<#field_type>();

                let request_query = RequestQuery(request.#field_name);
                let query_str = ruma_api::exports::serde_urlencoded::to_string(
                    request_query,
                )?;

                let query_opt: Option<&str> = if query_str.is_empty() {
                    None
                } else {
                    Some(&query_str)
                };

                url.set_query(query_opt);
            }
        } else if self.request.has_query_fields() {
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

        let extract_request_query = if self.request.query_map_field().is_some() {
            quote! {
                let request_query = match ruma_api::exports::serde_urlencoded::from_str(
                    &request.uri().query().unwrap_or("")
                ) {
                    Ok(query) => query,
                    Err(err) => {
                        return Err(
                            ruma_api::error::RequestDeserializationError::new(err, request).into()
                        );
                    }
                };
            }
        } else if self.request.has_query_fields() {
            quote! {
                let request_query: RequestQuery =
                    match ruma_api::exports::serde_urlencoded::from_str(
                        &request.uri().query().unwrap_or("")
                    ) {
                        Ok(query) => query,
                        Err(err) => {
                            return Err(
                                ruma_api::error::RequestDeserializationError::new(err, request)
                                    .into()
                            );
                        }
                    };
            }
        } else {
            TokenStream::new()
        };

        let parse_request_query = if let Some(field) = self.request.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");

            quote! {
                #field_name: request_query
            }
        } else {
            self.request.request_init_query_fields()
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

        let extract_request_headers = if self.request.has_header_fields() {
            quote! {
                let headers = request.headers();
            }
        } else {
            TokenStream::new()
        };

        let extract_request_body =
            if self.request.has_body_fields() || self.request.newtype_body_field().is_some() {
                quote! {
                    let request_body: <RequestBody as ruma_api::Outgoing>::Incoming =
                        match ruma_api::exports::serde_json::from_slice(request.body().as_slice()) {
                            Ok(body) => body,
                            Err(err) => {
                                return Err(
                                    ruma_api::error::RequestDeserializationError::new(err, request)
                                        .into()
                                );
                            }
                        };
                }
            } else {
                TokenStream::new()
            };

        let parse_request_headers = if self.request.has_header_fields() {
            self.request.parse_headers_from_request()
        } else {
            TokenStream::new()
        };

        let request_body = if let Some(field) = self.request.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote!(request.#field_name)
        } else if self.request.has_body_fields() || self.request.newtype_body_field().is_some() {
            let request_body_initializers = if let Some(field) = self.request.newtype_body_field() {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                quote! { (request.#field_name) }
            } else {
                let initializers = self.request.request_body_init_fields();
                quote! { { #initializers } }
            };

            quote! {
                {
                    let request_body = RequestBody #request_body_initializers;
                    ruma_api::exports::serde_json::to_vec(&request_body)?
                }
            }
        } else {
            quote!(Vec::new())
        };

        let parse_request_body = if let Some(field) = self.request.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! {
                #field_name: request_body.0,
            }
        } else if let Some(field) = self.request.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! {
                #field_name: request.into_body(),
            }
        } else {
            self.request.request_init_body_fields()
        };

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
                let response_body: <ResponseBody as ruma_api::Outgoing>::Incoming =
                    match ruma_api::exports::serde_json::from_slice(response.body().as_slice()) {
                        Ok(body) => body,
                        Err(err) => {
                            return Err(
                                ruma_api::error::ResponseDeserializationError::new(err, response)
                                    .into()
                            );
                        }
                    };
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

        let api = quote! {
            use ruma_api::exports::serde::de::Error as _;
            use ruma_api::exports::serde::Deserialize as _;
            use ruma_api::Endpoint as _;

            use std::convert::TryInto as _;

            #[doc = #request_doc]
            #request_type

            impl std::convert::TryFrom<ruma_api::exports::http::Request<Vec<u8>>> for #request_try_from_type {
                type Error = ruma_api::error::FromHttpRequestError;

                #[allow(unused_variables)]
                fn try_from(request: ruma_api::exports::http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
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

            impl std::convert::TryFrom<Request> for ruma_api::exports::http::Request<Vec<u8>> {
                type Error = ruma_api::error::IntoHttpError;

                #[allow(unused_mut, unused_variables)]
                fn try_from(request: Request) -> Result<Self, Self::Error> {
                    let metadata = Request::METADATA;

                    // Use dummy homeserver url which has to be overwritten in
                    // the calling code. Previously (with http::Uri) this was
                    // not required, but Url::parse only accepts absolute urls.
                    let mut url =
                        ruma_api::exports::url::Url::parse("http://invalid-host-please-change/")
                            .unwrap();

                    { #url_set_path }
                    { #url_set_querystring }

                    let mut http_request = ruma_api::exports::http::Request::new(#request_body);

                    *http_request.method_mut() = ruma_api::exports::http::Method::#method;
                    *http_request.uri_mut() = url.into_string().parse().unwrap();

                    { #add_headers_to_request }

                    Ok(http_request)
                }
            }

            #[doc = #response_doc]
            #response_type

            impl std::convert::TryFrom<Response> for ruma_api::exports::http::Response<Vec<u8>> {
                type Error = ruma_api::error::IntoHttpError;

                #[allow(unused_variables)]
                fn try_from(response: Response) -> Result<Self, Self::Error> {
                    let response = ruma_api::exports::http::Response::builder()
                        .header(ruma_api::exports::http::header::CONTENT_TYPE, "application/json")
                        #serialize_response_headers
                        .body(#body)
                        .unwrap();
                    Ok(response)
                }
            }

            impl std::convert::TryFrom<ruma_api::exports::http::Response<Vec<u8>>> for #response_try_from_type {
                type Error = ruma_api::error::FromHttpResponseError<#error>;

                #[allow(unused_variables)]
                fn try_from(
                    response: ruma_api::exports::http::Response<Vec<u8>>,
                ) -> Result<Self, Self::Error> {
                    if response.status().as_u16() < 400 {
                        #extract_response_headers

                        #typed_response_body_decl

                        Ok(Self {
                            #response_init_fields
                        })
                    } else {
                        match <#error as ruma_api::EndpointError>::try_from_response(response) {
                            Ok(err) => Err(ruma_api::error::ServerError::Known(err).into()),
                            Err(response_err) => Err(ruma_api::error::ServerError::Unknown(response_err).into())
                        }
                    }
                }
            }

            impl ruma_api::Endpoint for Request {
                type Response = Response;
                type ResponseError = #error;

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

pub struct RawResponse {
    pub response_kw: kw::response,
    pub fields: Vec<Field>,
}

impl Parse for RawResponse {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let response_kw = input.parse::<kw::response>()?;
        let fields;
        braced!(fields in input);

        Ok(Self {
            response_kw,
            fields: fields
                .parse_terminated::<Field, Token![,]>(Field::parse_named)?
                .into_iter()
                .collect(),
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
