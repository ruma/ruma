use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Field, FieldValue, Ident, Meta};

mod metadata;
mod request;
mod response;

use self::metadata::Metadata;
use self::request::Request;
use self::response::Response;

pub fn strip_serde_attrs(field: &Field) -> Field {
    let mut field = field.clone();

    field.attrs = field.attrs.into_iter().filter(|attr| {
        let meta = attr.interpret_meta()
            .expect("ruma_api! could not parse field attributes");

        let meta_list = match meta {
            Meta::List(meta_list) => meta_list,
            _ => return true,
        };

        if meta_list.ident == "serde" {
            return false;
        }

        true
    }).collect();

    field
}

pub struct Api {
    metadata: Metadata,
    request: Request,
    response: Response,
}

impl From<RawApi> for Api {
    fn from(raw_api: RawApi) -> Self {
        Api {
            metadata: raw_api.metadata.into(),
            request: raw_api.request.into(),
            response: raw_api.response.into(),
        }
    }
}

impl ToTokens for Api {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = &self.metadata.description;
        let method = Ident::new(self.metadata.method.as_ref(), Span::call_site());
        let name = &self.metadata.name;
        let path = &self.metadata.path;
        let rate_limited = &self.metadata.rate_limited;
        let requires_authentication = &self.metadata.requires_authentication;

        let request = &self.request;
        let request_types = quote! { #request };
        let response = &self.response;
        let response_types = quote! { #response };

        let extract_request_path = if self.request.has_path_fields() {
            quote! {
                let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();
            }
        } else {
            TokenStream::new()
        };

        let (set_request_path, parse_request_path) = if self.request.has_path_fields() {
            let path_str = path.as_str();

            assert!(path_str.starts_with('/'), "path needs to start with '/'");
            assert!(
                path_str.chars().filter(|c| *c == ':').count() == self.request.path_field_count(),
                "number of declared path parameters needs to match amount of placeholders in path"
            );

            let request_path_init_fields = self.request.request_path_init_fields();

            let mut set_tokens = quote! {
                let request_path = RequestPath {
                    #request_path_init_fields
                };

                // This `unwrap()` can only fail when the url is a
                // cannot-be-base url like `mailto:` or `data:`, which is not
                // the case for our placeholder url.
                let mut path_segments = url.path_segments_mut().unwrap();
            };

            let mut parse_tokens = TokenStream::new();

            for (i, segment) in path_str[1..].split('/').into_iter().enumerate() {
                set_tokens.append_all(quote! {
                    path_segments.push
                });

                if segment.starts_with(':') {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::new(path_var, Span::call_site());

                    set_tokens.append_all(quote! {
                        (&request_path.#path_var_ident.to_string());
                    });

                    let path_field = self.request.path_field(path_var)
                        .expect("expected request to have path field");
                    let ty = &path_field.ty;

                    parse_tokens.append_all(quote! {
                        #path_var_ident: {
                            let segment = path_segments.get(#i).unwrap().as_bytes();
                            let decoded =
                                ::url::percent_encoding::percent_decode(segment)
                                .decode_utf8_lossy();
                            #ty::deserialize(decoded.into_deserializer())
                                .map_err(|e: ::serde_json::error::Error| e)?
                        },
                    });
                } else {
                    set_tokens.append_all(quote! {
                        (#segment);
                    });
                }
            }

            (set_tokens, parse_tokens)
        } else {
            let set_tokens = quote! {
                url.set_path(metadata.path);
            };
            let parse_tokens = TokenStream::new();
            (set_tokens, parse_tokens)
        };

        let set_request_query = if self.request.has_query_fields() {
            let request_query_init_fields = self.request.request_query_init_fields();

            quote! {
                let request_query = RequestQuery {
                    #request_query_init_fields
                };

                url.set_query(Some(&::serde_urlencoded::to_string(request_query)?));
            }
        } else {
            TokenStream::new()
        };

        let extract_request_query = if self.request.has_query_fields() {
            quote! {
                let request_query: RequestQuery =
                    ::serde_urlencoded::from_str(&request.uri().query().unwrap_or(""))?;
            }
        } else {
            TokenStream::new()
        };

        let parse_request_query = if self.request.has_query_fields() {
            self.request.request_init_query_fields()
        } else {
            TokenStream::new()
        };

        let add_headers_to_request = if self.request.has_header_fields() {
            let mut header_tokens = quote! {
                let headers = http_request.headers_mut();
            };

            header_tokens.append_all(self.request.add_headers_to_request());

            header_tokens
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

        let parse_request_headers = if self.request.has_header_fields() {
            self.request.parse_headers_from_request()
        } else {
            TokenStream::new()
        };

        let create_http_request = if let Some(field) = self.request.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");

            quote! {
                let request_body = RequestBody(request.#field_name);

                let mut http_request = ::http::Request::new(::serde_json::to_vec(&request_body)?.into());
            }
        } else if self.request.has_body_fields() {
            let request_body_init_fields = self.request.request_body_init_fields();

            quote! {
                let request_body = RequestBody {
                    #request_body_init_fields
                };

                let mut http_request = ::http::Request::new(::serde_json::to_vec(&request_body)?.into());
            }
        } else {
            quote! {
                let mut http_request = ::http::Request::new(::hyper::Body::empty());
            }
        };

        let extract_request_body = if let Some(field) = self.request.newtype_body_field() {
            let ty = &field.ty;
            quote! {
                let request_body: #ty =
                    ::serde_json::from_slice(request.body().as_slice())?;
            }
        } else if self.request.has_body_fields() {
            quote! {
                let request_body: RequestBody =
                    ::serde_json::from_slice(request.body().as_slice())?;
            }
        } else {
            TokenStream::new()
        };

        let parse_request_body = if let Some(field) = self.request.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");

            quote! {
                #field_name: request_body,
            }
        } else if self.request.has_body_fields() {
            self.request.request_init_body_fields()
        } else {
            TokenStream::new()
        };

        let deserialize_response_body = if let Some(field) = self.response.newtype_body_field() {
            let field_type = &field.ty;

            quote! {
                let future_response = http_response.into_body()
                    .fold(Vec::new(), |mut vec, chunk| {
                        vec.extend(chunk.iter());
                        ::futures::future::ok::<_, ::hyper::Error>(vec)
                    })
                    .map_err(::ruma_api::Error::from)
                    .and_then(|data|
                              ::serde_json::from_slice::<#field_type>(data.as_slice())
                              .map_err(::ruma_api::Error::from)
                              .into_future()
                    )
            }
        } else if self.response.has_body_fields() {
            quote! {
                let future_response = http_response.into_body()
                    .fold(Vec::new(), |mut vec, chunk| {
                        vec.extend(chunk.iter());
                        ::futures::future::ok::<_, ::hyper::Error>(vec)
                    })
                    .map_err(::ruma_api::Error::from)
                    .and_then(|data|
                              ::serde_json::from_slice::<ResponseBody>(data.as_slice())
                              .map_err(::ruma_api::Error::from)
                              .into_future()
                    )
            }
        } else {
            quote! {
                let future_response = ::futures::future::ok(())
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

        let serialize_response_headers = self.response.apply_header_fields();

        let serialize_response_body = if self.response.has_body() {
            let body = self.response.to_body();
            quote! {
                .body(::hyper::Body::from(::serde_json::to_vec(&#body)?))
            }
        } else {
            quote! {
                .body(::hyper::Body::from("{}".as_bytes().to_vec()))
            }
        };

        tokens.append_all(quote! {
            #[allow(unused_imports)]
            use ::futures::{Future as _Future, IntoFuture as _IntoFuture, Stream as _Stream};
            use ::ruma_api::Endpoint as _RumaApiEndpoint;
            use ::serde::Deserialize;
            use ::serde::de::{Error as _SerdeError, IntoDeserializer};

            use ::std::convert::{TryInto as _TryInto};

            /// The API endpoint.
            #[derive(Debug)]
            pub struct Endpoint;

            #request_types

            impl ::std::convert::TryFrom<::http::Request<Vec<u8>>> for Request {
                type Error = ::ruma_api::Error;

                #[allow(unused_variables)]
                fn try_from(request: ::http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
                    #extract_request_path
                    #extract_request_query
                    #extract_request_headers
                    #extract_request_body

                    Ok(Request {
                        #parse_request_path
                        #parse_request_query
                        #parse_request_headers
                        #parse_request_body
                    })
                }
            }

            impl ::futures::future::FutureFrom<::http::Request<::hyper::Body>> for Request {
                type Future = Box<_Future<Item = Self, Error = Self::Error> + Send>;
                type Error = ::ruma_api::Error;

                #[allow(unused_variables)]
                fn future_from(request: ::http::Request<::hyper::Body>) -> Self::Future {
                    let (parts, body) = request.into_parts();
                    let future = body.from_err().fold(Vec::new(), |mut vec, chunk| {
                        vec.extend(chunk.iter());
                        ::futures::future::ok::<_, Self::Error>(vec)
                    }).and_then(|body| {
                        ::http::Request::from_parts(parts, body)
                            .try_into()
                            .into_future()
                            .from_err()
                    });
                    Box::new(future)
                }
            }

            impl ::std::convert::TryFrom<Request> for ::http::Request<::hyper::Body> {
                type Error = ::ruma_api::Error;

                #[allow(unused_mut, unused_variables)]
                fn try_from(request: Request) -> Result<Self, Self::Error> {
                    let metadata = Endpoint::METADATA;

                    // Use dummy homeserver url which has to be overwritten in
                    // the calling code. Previously (with http::Uri) this was
                    // not required, but Url::parse only accepts absolute urls.
                    let mut url = ::url::Url::parse("http://invalid-host-please-change/").unwrap();

                    { #set_request_path }
                    { #set_request_query }

                    #create_http_request

                    *http_request.method_mut() = ::http::Method::#method;
                    *http_request.uri_mut() = url.into_string().parse().unwrap();

                    { #add_headers_to_request }

                    Ok(http_request)
                }
            }

            #response_types

            impl ::std::convert::TryFrom<Response> for ::http::Response<::hyper::Body> {
                type Error = ::ruma_api::Error;

                #[allow(unused_variables)]
                fn try_from(response: Response) -> Result<Self, Self::Error> {
                    let response = ::http::Response::builder()
                        .header(::http::header::CONTENT_TYPE, "application/json")
                        #serialize_response_headers
                        #serialize_response_body
                        .unwrap();
                    Ok(response)
                }
            }

            impl ::futures::future::FutureFrom<::http::Response<::hyper::Body>> for Response {
                type Future = Box<_Future<Item = Self, Error = Self::Error> + Send>;
                type Error = ::ruma_api::Error;

                #[allow(unused_variables)]
                fn future_from(http_response: ::http::Response<::hyper::Body>) -> Self::Future {
                    if http_response.status().is_success() {
                        #extract_response_headers

                        #deserialize_response_body
                        .and_then(move |response_body| {
                            let response = Response {
                                #response_init_fields
                            };

                            Ok(response)
                        });

                        Box::new(future_response)
                    } else {
                        Box::new(::futures::future::err(::ruma_api::Error::StatusCode(http_response.status().clone())))
                    }
                }
            }

            impl ::ruma_api::Endpoint for Endpoint {
                type Request = Request;
                type Response = Response;

                const METADATA: ::ruma_api::Metadata = ::ruma_api::Metadata {
                    description: #description,
                    method: ::http::Method::#method,
                    name: #name,
                    path: #path,
                    rate_limited: #rate_limited,
                    requires_authentication: #requires_authentication,
                };
            }
        });
    }
}

type ParseMetadata = Punctuated<FieldValue, Token![,]>;
type ParseFields = Punctuated<Field, Token![,]>;

pub struct RawApi {
    pub metadata: Vec<FieldValue>,
    pub request: Vec<Field>,
    pub response: Vec<Field>,
}

impl Synom for RawApi {
    named!(parse -> Self, do_parse!(
        custom_keyword!(metadata) >>
        metadata: braces!(ParseMetadata::parse_terminated) >>
        custom_keyword!(request) >>
        request: braces!(call!(ParseFields::parse_terminated_with, Field::parse_named)) >>
        custom_keyword!(response) >>
        response: braces!(call!(ParseFields::parse_terminated_with, Field::parse_named)) >>
        (RawApi {
            metadata: metadata.1.into_iter().collect(),
            request: request.1.into_iter().collect(),
            response: response.1.into_iter().collect(),
        })
    ));
}
