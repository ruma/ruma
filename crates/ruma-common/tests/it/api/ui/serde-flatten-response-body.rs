#![allow(unexpected_cfgs)]

use ruma_common::{
    api::{auth_scheme::NoAuthentication, request, response},
    metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomResponseBody {
    pub bar: String,
}

metadata! {
    method: GET, // An `http::Method` constant. No imports required.
    rate_limited: false,
    authentication: NoAuthentication,
    history: {
        unstable => "/_matrix/some/endpoint",
    }
}

#[request]
pub struct Request;

#[response]
pub struct Response {
    #[serde(flatten)]
    pub foo: CustomResponseBody,
}

fn main() {}
