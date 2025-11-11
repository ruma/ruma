#![cfg(feature = "client")]

use http::HeaderMap;
use ruma_client_api::discovery::discover_homeserver;
use ruma_common::api::{OutgoingRequest as _, auth_scheme::SendAccessToken};

#[test]
fn get_request_headers() {
    let req: http::Request<Vec<u8>> = discover_homeserver::Request::new()
        .try_into_http_request("https://homeserver.tld", SendAccessToken::None, ())
        .unwrap();

    assert_eq!(*req.headers(), HeaderMap::default());
}
