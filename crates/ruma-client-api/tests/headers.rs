#![cfg(feature = "client")]

use http::HeaderMap;
use ruma_api::{OutgoingRequest as _, SendAccessToken};
use ruma_client_api::unversioned::discover_homeserver;

#[test]
fn get_request_headers() {
    let req: http::Request<Vec<u8>> = discover_homeserver::Request::new()
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::None,
            ruma_api::EndpointPath::PreferStable,
        )
        .unwrap();

    assert_eq!(*req.headers(), HeaderMap::default());
}
