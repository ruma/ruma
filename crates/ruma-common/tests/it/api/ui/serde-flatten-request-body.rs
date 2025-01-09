#![allow(unexpected_cfgs)]

use ruma_common::{
    api::{request, response, Metadata},
    metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRequestBody {
    pub bar: String,
}

const METADATA: Metadata = metadata! {
    method: POST, // An `http::Method` constant. No imports required.
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/some/endpoint",
    }
};

#[request]
pub struct Request {
    #[serde(flatten)]
    pub foo: CustomRequestBody,
}

#[response]
pub struct Response;

fn main() {}
