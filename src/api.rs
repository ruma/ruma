use quote::{ToTokens, Tokens};

use metadata::Metadata;
use parse::Entry;
use request::{Request, RequestField};
use response::{Response, ResponseField};

#[derive(Debug)]
pub struct Api {
    metadata: Metadata,
    request: Request,
    response: Response,
}

impl Api {
    pub fn output(&self) -> Tokens {
        let description = &self.metadata.description;
        let method = &self.metadata.method;
        let name = &self.metadata.name;
        let path = &self.metadata.path;
        let rate_limited = &self.metadata.rate_limited;
        let requires_authentication = &self.metadata.requires_authentication;

        let request_types = self.generate_request_types();
        let response_types = self.generate_response_types();

        quote! {
            use std::convert::TryFrom;

            /// The API endpoint.
            #[derive(Debug)]
            pub struct Endpoint;

            #request_types

            impl TryFrom<Request> for ::hyper::Request {
                type Error = ();

                fn try_from(request: Request) -> Result<Self, Self::Error> {
                    Ok(
                        ::hyper::Request::new(
                            ::hyper::#method,
                            "/".parse().expect("failed to parse request URI"),
                        )
                    )
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
        }
    }

    fn generate_request_types(&self) -> Tokens {
        let mut tokens = quote! {
            /// Data for a request to this API endpoint.
            #[derive(Debug)]
            pub struct Request
        };

        if self.request.fields.len() == 0 {
            tokens.append(";");
        } else {
            tokens.append("{");

            for request_field in self.request.fields.iter() {
                match *request_field {
                    RequestField::Body(ref field) => field.to_tokens(&mut tokens),
                    RequestField::Header(_, ref field) => field.to_tokens(&mut tokens),
                    RequestField::Path(_, ref field) => field.to_tokens(&mut tokens),
                    RequestField::Query(ref field) => field.to_tokens(&mut tokens),
                }
            }

            tokens.append("}");
        }

        tokens
    }

    fn generate_response_types(&self) -> Tokens {
        let mut tokens = quote! {
            /// Data in the response from this API endpoint.
            #[derive(Debug)]
            pub struct Response
        };

        if self.response.fields.len() == 0 {
            tokens.append(";");
        } else {
            tokens.append("{");

            for response in self.response.fields.iter() {
                match *response {
                    ResponseField::Body(ref field) => field.to_tokens(&mut tokens),
                    ResponseField::Header(_, ref field) => field.to_tokens(&mut tokens),
                }
            }

            tokens.append("}");
        }

        tokens
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


