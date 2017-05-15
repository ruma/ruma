use quote::{ToTokens, Tokens};

use metadata::Metadata;
use parse::Entry;
use request::Request;
use response::Response;

#[derive(Debug)]
pub struct Api {
    metadata: Metadata,
    request: Request,
    response: Response,
}

impl ToTokens for Api {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let description = &self.metadata.description;
        let method = &self.metadata.method;
        let name = &self.metadata.name;
        let path = &self.metadata.path;
        let rate_limited = &self.metadata.rate_limited;
        let requires_authentication = &self.metadata.requires_authentication;

        let request_types = {
            let mut tokens = Tokens::new();
            self.request.to_tokens(&mut tokens);
            tokens
        };
        let response_types = {
            let mut tokens = Tokens::new();
            self.response.to_tokens(&mut tokens);
            tokens
        };

        let add_body_to_request = if self.request.has_body_fields() {
            let request_body_init_fields = self.request.request_body_init_fields();

            quote! {
                let request_body = RequestBody {
                    #request_body_init_fields
                };

                hyper_request.set_body(::serde_json::to_vec(&request_body)?);
            }
        } else {
            Tokens::new()
        };

        let deserialize_response_body = if self.response.has_body_fields() {
            let mut tokens = Tokens::new();

            tokens.append(quote! {
                let future_response = hyper_response.body()
                    .fold::<_, _, Result<_, ::std::io::Error>>(Vec::new(), |mut bytes, chunk| {
                        bytes.write_all(&chunk)?;

                        Ok(bytes)
                    })
                    .map_err(::ruma_api::Error::from)
                    .and_then(|bytes| {
                        ::serde_json::from_slice::<ResponseBody>(bytes.as_slice())
                            .map_err(::ruma_api::Error::from)
                    })
            });

            tokens.append(".and_then(|response_body| {");

            tokens
        } else {
            let mut tokens = Tokens::new();

            tokens.append(quote! {
                let future_response = ::futures::future::ok(())
            });

            tokens.append(".and_then(|_| {");

            tokens
        };

        let mut closure_end = Tokens::new();
        closure_end.append("});");

        let response_init_fields = if self.response.has_fields() {
            self.response.init_fields()
        } else {
            Tokens::new()
        };

        tokens.append(quote! {
            #[allow(unused_imports)]
            use std::io::Write as _Write;

            #[allow(unused_imports)]
            use ::futures::{Future as _Future, Stream as _Stream};
            use ::ruma_api::Endpoint as _RumaApiEndpoint;

            /// The API endpoint.
            #[derive(Debug)]
            pub struct Endpoint;

            #request_types

            impl ::std::convert::TryFrom<Request> for ::hyper::Request {
                type Error = ::ruma_api::Error;

                #[allow(unused_mut, unused_variables)]
                fn try_from(request: Request) -> Result<Self, Self::Error> {
                    let metadata = Endpoint::METADATA;

                    let mut hyper_request = ::hyper::Request::new(
                        metadata.method,
                        metadata.path.parse()?,
                    );

                    #add_body_to_request

                    Ok(hyper_request)
                }
            }

            #response_types

            impl ::futures::future::FutureFrom<::hyper::Response> for Response {
                type Future = Box<_Future<Item = Self, Error = Self::Error>>;
                type Error = ::ruma_api::Error;

                #[allow(unused_variables)]
                fn future_from(hyper_response: ::hyper::Response)
                -> Box<_Future<Item = Self, Error = Self::Error>> {
                    #deserialize_response_body

                    let response = Response {
                        #response_init_fields
                    };

                    Ok(response)
                    #closure_end

                    Box::new(future_response)
                }
            }

            impl ::ruma_api::Endpoint for Endpoint {
                type Request = Request;
                type Response = Response;

                const METADATA: ::ruma_api::Metadata = ::ruma_api::Metadata {
                    description: #description,
                    method: ::hyper::#method,
                    name: #name,
                    path: #path,
                    rate_limited: #rate_limited,
                    requires_authentication: #requires_authentication,
                };
            }
        });
    }
}

impl From<Vec<Entry>> for Api {
    fn from(entries: Vec<Entry>) -> Api {
        if entries.len() != 3 {
            panic!("ruma_api! expects 3 blocks: metadata, request, and response");
        }

        let mut metadata = None;
        let mut request = None;
        let mut response = None;

        for entry in entries {
            match entry {
                Entry::Metadata(fields) => metadata = Some(Metadata::from(fields)),
                Entry::Request(fields) => request = Some(Request::from(fields)),
                Entry::Response(fields) => response = Some(Response::from(fields)),
            }
        }

        Api {
            metadata: metadata.expect("ruma_api! is missing metadata"),
            request: request.expect("ruma_api! is missing request"),
            response: response.expect("ruma_api! is missing response"),
        }
    }
}
