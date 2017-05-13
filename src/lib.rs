#![feature(proc_macro)]

extern crate hyper;
extern crate proc_macro;
extern crate quote;
extern crate ruma_api;
extern crate syn;

use proc_macro::TokenStream;

use hyper::Method;
use quote::{ToTokens, Tokens};
use syn::{ExprKind, Item, ItemKind, Lit, parse_items};

#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let items = parse_items(&input.to_string())
        .expect("failed to parse input");

    let api = Api::from(items);

    api.output().parse().expect("failed to parse output")
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

impl From<Vec<Item>> for Api {
    fn from(items: Vec<Item>) -> Api {
        if items.len() != 3 {
            panic!("ruma_api! expects exactly three items: const METADATA, struct Request, and struct Response");
        }

        let mut metadata = None;
        let mut request = None;
        let mut response = None;

        for item in items {
            match &item.ident.to_string()[..] {
                "METADATA" => metadata = Some(Metadata::from(item)),
                "Request" => request = Some(Request::from(item)),
                "Response" => response = Some(Response::from(item)),
                other => panic!("ruma_api! found unexpected item: {}", other),
            }
        }

        if metadata.is_none() {
            panic!("ruma_api! requires item: const METADATA");
        }

        if request.is_none() {
            panic!("ruma_api! requires item: struct Request");
        }

        if response.is_none() {
            panic!("ruma_api! requires item: struct Response");
        }

        Api {
            metadata: metadata.expect("metadata is missing"),
            request: request.expect("request is missing"),
            response: response.expect("response is missing"),
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

impl From<Item> for Metadata {
    fn from(item: Item) -> Self {
        let expr = match item.node {
            ItemKind::Const(_, expr)  => expr,
            _ => panic!("ruma_api! expects METADATA to be a const item"),
        };

        let field_values = match expr.node {
            ExprKind::Struct(_, field_values, _) => field_values,
            _ => panic!("ruma_api! expects METADATA to be a Metadata struct"),
        };

        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut requires_authentication = None;

        for field_value in field_values {
            match &field_value.ident.to_string()[..] {
                "description" => {
                    match field_value.expr.node {
                        ExprKind::Lit(Lit::Str(value, _)) => description = Some(tokens_for(value)),
                        _ => panic!("ruma_api! expects metadata description to be a &'static str"),
                    }
                }
                "method" => {
                    match field_value.expr.node {
                        ExprKind::Path(_, value) => method = Some(tokens_for(value)),
                        _ => panic!("ruma_api! expects metadata method to be a path"),
                    }
                }
                "name" => {
                    match field_value.expr.node {
                        ExprKind::Lit(Lit::Str(value, _)) => name = Some(tokens_for(value)),
                        _ => panic!("ruma_api! expects metadata name to be a &'static str"),
                    }
                }
                "path" => {
                    match field_value.expr.node {
                        ExprKind::Lit(Lit::Str(value, _)) => path = Some(tokens_for(value)),
                        _ => panic!("ruma_api! expects metadata path to be a &'static str"),
                    }
                }
                "rate_limited" => {
                    match field_value.expr.node {
                        ExprKind::Lit(Lit::Bool(value)) => rate_limited = Some(tokens_for(value)),
                        _ => panic!("ruma_api! expects metadata rate_limited to be a bool"),
                    }
                }
                "requires_authentication" => {
                    match field_value.expr.node {
                        ExprKind::Lit(Lit::Bool(value)) =>  {
                            requires_authentication = Some(tokens_for(value));
                        }
                        _ => panic!("ruma_api! expects metadata requires_authentication to be a bool"),
                    }
                }
                other => panic!("ruma_api! found unexpected metadata field: {}", other),
            }
        }

        if description.is_none() {
            panic!("ruma_api! metadata requires field: description");
        }

        if method.is_none() {
            panic!("ruma_api! metadata requires field: method");
        }

        if name.is_none() {
            panic!("ruma_api! metadata requires field: name");
        }

        if path.is_none() {
            panic!("ruma_api! metadata requires field: path");
        }

        if rate_limited.is_none() {
            panic!("ruma_api! metadata requires field: rate_limited");
        }

        if requires_authentication.is_none() {
            panic!("ruma_api! metadata requires field: requires_authentication");
        }

        Metadata {
            description: description.expect("description is missing"),
            method: method.expect("method is missing"),
            name: name.expect("name is missing"),
            path: path.expect("path is missing"),
            rate_limited: rate_limited.expect("rate limited is missing"),
            requires_authentication: requires_authentication.expect("requires_authentication is missing"),
        }
    }
}

struct Request;

impl From<Item> for Request {
    fn from(item: Item) -> Self {
        Request
        // panic!("ruma_api! could not parse Request");
    }
}

struct Response;

impl From<Item> for Response {
    fn from(item: Item) -> Self {
        Response
        // panic!("ruma_api! could not parse Response");
    }
}

fn tokens_for<T>(value: T) -> Tokens where T: ToTokens {
    let mut tokens = Tokens::new();
    value.to_tokens(&mut tokens);
    tokens
}
