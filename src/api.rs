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

                hyper_request.set_body(
                    ::serde_json::to_vec(&request_body)
                        .expect("failed to serialize request body to JSON")
                );
            }
        } else {
            Tokens::new()
        };

        tokens.append(quote! {
            use std::convert::TryFrom;

            /// The API endpoint.
            #[derive(Debug)]
            pub struct Endpoint;

            #request_types

            impl TryFrom<Request> for ::hyper::Request {
                type Error = ();

                fn try_from(request: Request) -> Result<Self, Self::Error> {
                    let mut hyper_request = ::hyper::Request::new(
                        ::hyper::#method,
                        "/".parse().expect("failed to parse request URI"),
                    );

                    #add_body_to_request

                    Ok(hyper_request)
                }
            }

            #response_types

            impl TryFrom<::hyper::Response> for Response {
                type Error = ();

                fn try_from(hyper_response: ::hyper::Response) -> Result<Self, Self::Error> {
                    Ok(Response)
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
