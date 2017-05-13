#![feature(proc_macro)]

extern crate proc_macro;
extern crate quote;
extern crate ruma_api;
extern crate syn;
#[macro_use] extern crate synom;

use proc_macro::TokenStream;

use quote::{ToTokens, Tokens};
use syn::{Expr, Field, Ident, Item};

use parse::{Entry, parse_entries};

mod parse;

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
        Tokens::new()
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

struct Request;

impl From<Vec<Field>> for Request {
    fn from(fields: Vec<Field>) -> Self {
        Request
    }
}

struct Response;

impl From<Vec<Field>> for Response {
    fn from(fields: Vec<Field>) -> Self {
        Response
    }
}

/// Helper method for turning a value into tokens.
fn tokens_for<T>(value: T) -> Tokens where T: ToTokens {
    let mut tokens = Tokens::new();

    value.to_tokens(&mut tokens);

    tokens
}
