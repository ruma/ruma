//! Crate `ruma-api-macros` provides a procedural macro for easily generating `ruma-api` endpoints.

#![deny(missing_debug_implementations)]
#![feature(proc_macro)]
#![recursion_limit="128"]

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate ruma_api;
extern crate syn;
#[macro_use] extern crate synom;

use proc_macro::TokenStream;

use quote::{ToTokens, Tokens};
use syn::{Expr, Field, Ident};

use parse::{Entry, parse_entries};

mod parse;

/// Generates a `ruma-api` endpoint.
#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let entries = parse_entries(&input.to_string()).expect("ruma_api! failed to parse input");

    let api = Api::from(entries);

    api.output().parse().expect("ruma_api! failed to parse output as a TokenStream")
}

struct Api {
    metadata: Metadata,
    request: Request,
    response: Response,
}

impl Api {
    fn output(&self) -> Tokens {
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
            pub struct Request;
        };

        tokens
    }

    fn generate_response_types(&self) -> Tokens {
        let mut tokens = quote! {
            /// Data in the response from this API endpoint.
            #[derive(Debug)]
            pub struct Response;
        };

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

struct Metadata {
    description: Tokens,
    method: Tokens,
    name: Tokens,
    path: Tokens,
    rate_limited: Tokens,
    requires_authentication: Tokens,
}

impl From<Vec<(Ident, Expr)>> for Metadata {
    fn from(fields: Vec<(Ident, Expr)>) -> Self {
        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut requires_authentication = None;

        for field in fields {
            let (identifier, expression) = field;

            if identifier == Ident::new("description") {
                description = Some(tokens_for(expression));
            } else if identifier == Ident::new("method") {
                method = Some(tokens_for(expression));
            } else if identifier == Ident::new("name") {
                name = Some(tokens_for(expression));
            } else if identifier == Ident::new("path") {
                path = Some(tokens_for(expression));
            } else if identifier == Ident::new("rate_limited") {
                rate_limited = Some(tokens_for(expression));
            } else if identifier == Ident::new("requires_authentication") {
                requires_authentication = Some(tokens_for(expression));
            } else {
                panic!("ruma_api! metadata included unexpected field: {}", identifier);
            }
        }

        Metadata {
            description: description.expect("ruma_api! metadata is missing description"),
            method: method.expect("ruma_api! metadata is missing method"),
            name: name.expect("ruma_api! metadata is missing name"),
            path: path.expect("ruma_api! metadata is missing path"),
            rate_limited: rate_limited.expect("ruma_api! metadata is missing rate_limited"),
            requires_authentication: requires_authentication
                .expect("ruma_api! metadata is missing requires_authentication"),
        }
    }
}

struct Request {
    fields: Vec<Field>,
    body_fields: Vec<usize>,
    header_fields: Vec<usize>,
    path_fields: Vec<usize>,
    query_string_fields: Vec<usize>,
}

impl From<Vec<Field>> for Request {
    fn from(fields: Vec<Field>) -> Self {
        Request {
            fields: fields,
            body_fields: vec![],
            header_fields: vec![],
            path_fields: vec![],
            query_string_fields: vec![],
        }
    }
}

struct Response {
    fields: Vec<Field>,
    body_fields: Vec<usize>,
    header_fields: Vec<usize>,
}

impl From<Vec<Field>> for Response {
    fn from(fields: Vec<Field>) -> Self {
        Response {
            fields: fields,
            body_fields: vec![],
            header_fields: vec![],
        }
    }
}

/// Helper method for turning a value into tokens.
fn tokens_for<T>(value: T) -> Tokens where T: ToTokens {
    let mut tokens = Tokens::new();

    value.to_tokens(&mut tokens);

    tokens
}
