#![allow(unexpected_cfgs)]

use ruma_common::{
    api::{request, response, Metadata},
    metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomResponseBody {
    pub bar: String,
}

const METADATA: Metadata = metadata! {
    method: GET, // An `http::Method` constant. No imports required.
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/some/endpoint",
    }
};

#[request]
pub struct Request;

#[response]
pub struct Response {
    #[serde(flatten)]
    pub foo: CustomResponseBody,
}

fn main() {}
