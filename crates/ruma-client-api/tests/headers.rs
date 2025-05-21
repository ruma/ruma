#![cfg(feature = "client")]

use http::HeaderMap;
use ruma_client_api::discovery::discover_homeserver;
use ruma_common::api::{MatrixVersion, OutgoingRequest as _, SendAccessToken, SupportedVersions};

#[test]
fn get_request_headers() {
    let supported =
        SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };
    let req: http::Request<Vec<u8>> = discover_homeserver::Request::new()
        .try_into_http_request("https://homeserver.tld", SendAccessToken::None, &supported)
        .unwrap();

    assert_eq!(*req.headers(), HeaderMap::default());
}
