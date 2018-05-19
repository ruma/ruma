use quote::{ToTokens, Tokens};
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

        if meta_list.ident.as_ref() == "serde" {
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
    fn to_tokens(&self, tokens: &mut Tokens) {
        let description = &self.metadata.description;
        let method = Ident::from(self.metadata.method.as_ref());
        let name = &self.metadata.name;
        let path = &self.metadata.path;
        let rate_limited = &self.metadata.rate_limited;
        let requires_authentication = &self.metadata.requires_authentication;

        let request = &self.request;
        let request_types = quote! { #request };
        let response = &self.response;
        let response_types = quote! { #response };

        let set_request_path = if self.request.has_path_fields() {
            let path_str = path.as_str();

            assert!(path_str.starts_with('/'), "path needs to start with '/'");
            assert!(
                path_str.chars().filter(|c| *c == ':').count() == self.request.path_field_count(),
                "number of declared path parameters needs to match amount of placeholders in path"
            );

            let request_path_init_fields = self.request.request_path_init_fields();

            let mut tokens = quote! {
                let request_path = RequestPath {
                    #request_path_init_fields
                };

                // This `unwrap()` can only fail when the url is a
                // cannot-be-base url like `mailto:` or `data:`, which is not
                // the case for our placeholder url.
                let mut path_segments = url.path_segments_mut().unwrap();
            };

            for segment in path_str[1..].split('/') {
                tokens.append_all(quote! {
                    path_segments.push
                });

                if segment.starts_with(':') {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::from(path_var);

                    tokens.append_all(quote! {
                        (&request_path.#path_var_ident.to_string());
                    });
                } else {
                    tokens.append_all(quote! {
                        (#segment);
                    });
                }
            }

            tokens
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

                url.set_query(Some(&::serde_urlencoded::to_string(request_query)?));
            }
        } else {
            Tokens::new()
        };

        let add_headers_to_request = if self.request.has_header_fields() {
            let mut header_tokens = quote! {
                let headers = http_request.headers_mut();
            };

            header_tokens.append_all(self.request.add_headers_to_request());

            header_tokens
        } else {
            Tokens::new()
        };

        let create_http_request = if let Some(field) = self.request.newtype_body_field() {
            let field_name = field.ident.expect("expected field to have an identifier");

            quote! {
                let request_body = RequestBody(request.#field_name);

                let mut http_request = ::http::Request::new(::serde_json::to_vec(&request_body)?);
            }
        } else if self.request.has_body_fields() {
            let request_body_init_fields = self.request.request_body_init_fields();

            quote! {
                let request_body = RequestBody {
                    #request_body_init_fields
                };

                let mut http_request = ::http::Request::new(::serde_json::to_vec(&request_body)?);
            }
        } else {
            quote! {
                let mut http_request = ::http::Request::new(Vec::with_capacity(0));
            }
        };

        let deserialize_response_body = if let Some(field) = self.response.newtype_body_field() {
            let field_type = &field.ty;

            quote! {
                let future_response =
                    ::serde_json::from_slice::<#field_type>(http_response.body().as_slice())
                        .into_future()
                        .map_err(::ruma_api::Error::from)
            }
        } else if self.response.has_body_fields() {
            quote! {
                let future_response =
                    ::serde_json::from_slice::<ResponseBody>(http_response.body().as_slice())
                        .into_future()
                        .map_err(::ruma_api::Error::from)
            }
        } else {
            quote! {
                let future_response = ::futures::future::ok(())
            }
        };

        let extract_headers = if self.response.has_header_fields() {
            quote! {
                let mut headers = http_response.headers().clone();
            }
        } else {
            Tokens::new()
        };

        let response_init_fields = if self.response.has_fields() {
            self.response.init_fields()
        } else {
            Tokens::new()
        };

        tokens.append_all(quote! {
            #[allow(unused_imports)]
            use ::futures::{Future as _Future, IntoFuture as _IntoFuture};
            use ::ruma_api::Endpoint as _RumaApiEndpoint;

            /// The API endpoint.
            #[derive(Debug)]
            pub struct Endpoint;

            #request_types

            impl ::std::convert::TryFrom<Request> for ::http::Request<Vec<u8>> {
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

            impl ::futures::future::FutureFrom<::http::Response<Vec<u8>>> for Response {
                type Future = Box<_Future<Item = Self, Error = Self::Error>>;
                type Error = ::ruma_api::Error;

                #[allow(unused_variables)]
                fn future_from(http_response: ::http::Response<Vec<u8>>)
                -> Box<_Future<Item = Self, Error = Self::Error>> {
                    if http_response.status().is_success() {
                        #extract_headers

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

            impl ::ruma_api::Endpoint<Vec<u8>, Vec<u8>> for Endpoint {
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
