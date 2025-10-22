#![cfg(feature = "client")]

use js_int::uint;
use ruma_common::{api::OutgoingRequest, owned_server_name, MilliSecondsSinceUnixEpoch};
use ruma_federation_api::{
    authentication::{ServerSignaturesInput, XMatrix},
    transactions::send_transaction_message,
};
use ruma_signatures::Ed25519KeyPair;

static PKCS8_ED25519_DER: &[u8] = include_bytes!("../../../ruma-signatures/tests/keys/ed25519.der");

#[test]
fn server_signatures_add_authentication() {
    let key_pair = Ed25519KeyPair::from_der(PKCS8_ED25519_DER, "1".to_owned()).unwrap();
    let origin = owned_server_name!("origin.local");
    let destination = owned_server_name!("destination.local");

    let request = send_transaction_message::v1::Request::new(
        "12345".into(),
        origin.clone(),
        MilliSecondsSinceUnixEpoch(uint!(1_000_000)),
    );

    let http_request = request
        .try_into_http_request::<Vec<u8>>(
            "https://destination.local",
            ServerSignaturesInput::new(origin.clone(), destination.clone(), &key_pair),
            (),
        )
        .unwrap();

    let authorization_header = http_request.headers().get(http::header::AUTHORIZATION).unwrap();
    let xmatrix = XMatrix::try_from(authorization_header).unwrap();

    assert_eq!(xmatrix.origin, origin);
    assert_eq!(xmatrix.destination, Some(destination));
    assert_eq!(xmatrix.key, "ed25519:1");
    assert_eq!(
        xmatrix.sig.encode(),
        "hyxb9TkzuQtnqYUo2nOQLhhsWs8yzrObbfBWKGCA0GZnHtniTfY8pTFPu1BmW3O47rRCm2tRODUaJXIYZeJFCQ"
    );
}
