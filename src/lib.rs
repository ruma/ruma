#![feature(proc_macro)]

extern crate proc_macro;
extern crate quote;
extern crate ruma_api;
extern crate syn;
#[macro_use] extern crate synom;

use proc_macro::TokenStream;

use quote::Tokens;
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
        Api {
            metadata: Metadata {
                description: Tokens::new(),
                method: Tokens::new(),
                name: Tokens::new(),
                path: Tokens::new(),
                rate_limited: Tokens::new(),
                requires_authentication: Tokens::new(),
            },
            request: Request,
            response: Response,
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
