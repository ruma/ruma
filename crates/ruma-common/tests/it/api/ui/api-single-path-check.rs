#![allow(unexpected_cfgs)]

use ruma_common::{
    api::{Metadata, auth_scheme::AccessToken, path_builder::PathBuilder, request, response},
    metadata,
};

metadata! {
    method: PUT,
    rate_limited: true,
    authentication: AccessToken,
    path: "/_matrix/other/v1/endpoint/{baz}",
}

/// Request type for the `other_endpoint` endpoint.
#[request]
pub struct Request {
    // This value will be inserted into the request's URL in place of the
    // "{baz}" path component.
    #[ruma_api(path)]
    pub baz: String,

    // This value represents the full query string of the request's URL.
    #[ruma_api(query_all)]
    pub bar: Query,

    // This is a non-JSON body.
    #[ruma_api(raw_body)]
    pub body: Vec<u8>,
}

/// The query parameters of the `other_endpoint` endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Query {
    pub foo: String,
}

/// Response type for the `other_endpoint` endpoint.
#[response]
pub struct Response {
    // This is a non-JSON body.
    #[ruma_api(raw_body)]
    pub body: Vec<u8>,
}

fn main() {
    assert_eq!(
        Request::PATH_BUILDER.all_paths().collect::<Vec<_>>(),
        &["/_matrix/other/v1/endpoint/{baz}"],
    );

    let path = Request::PATH_BUILDER.select_path(()).unwrap();
    assert_eq!(path, "/_matrix/other/v1/endpoint/{baz}");
}
