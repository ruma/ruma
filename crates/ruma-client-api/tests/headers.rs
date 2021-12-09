#![cfg(feature = "client")]

use http::HeaderMap;
use ruma_client_api::discovery::discover_homeserver;
use ruma_common::api::{MatrixVersion, OutgoingRequest as _, SendAccessToken};

#[test]
fn get_request_headers() {
    let req = discover_homeserver::Request::new()
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::None,
            &[MatrixVersion::V1_1],
        )
        .unwrap();

    assert_eq!(*req.headers(), HeaderMap::default());
}
