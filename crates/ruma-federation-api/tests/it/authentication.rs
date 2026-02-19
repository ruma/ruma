#![cfg(all(feature = "client", feature = "server"))]

use js_int::uint;
use ruma_common::{
    MilliSecondsSinceUnixEpoch,
    api::{OutgoingRequest, auth_scheme::AuthScheme},
    serde::Base64,
    server_name,
};
use ruma_federation_api::{
    authentication::{ServerSignatures, ServerSignaturesInput},
    transactions::send_transaction_message,
};
use ruma_signatures::{Ed25519KeyPair, PublicKeyMap, PublicKeySet};

static PKCS8_ED25519_DER: &[u8] =
    include_bytes!("../../../ruma-signatures/tests/it/keys/ed25519.der");

#[test]
fn server_signatures_roundtrip() {
    let key_pair = Ed25519KeyPair::from_der(PKCS8_ED25519_DER, "1".to_owned()).unwrap();
    let origin = server_name!("origin.local");
    let destination = server_name!("destination.local");

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

    let xmatrix = ServerSignatures::extract_authentication(&http_request).unwrap();

    assert_eq!(xmatrix.origin, origin);
    assert_eq!(xmatrix.destination.as_ref(), Some(&destination));
    assert_eq!(xmatrix.key, "ed25519:1");
    assert_eq!(
        xmatrix.sig.encode(),
        "hyxb9TkzuQtnqYUo2nOQLhhsWs8yzrObbfBWKGCA0GZnHtniTfY8pTFPu1BmW3O47rRCm2tRODUaJXIYZeJFCQ"
    );

    let public_key_set =
        PublicKeySet::from([("ed25519:1".to_owned(), Base64::new(key_pair.public_key().to_vec()))]);
    let public_key_map = PublicKeyMap::from([(origin.to_string(), public_key_set)]);

    // With invalid destination.
    xmatrix
        .verify_request(&http_request, &server_name!("invalid.local"), &public_key_map)
        .unwrap_err();

    // With valid destination.
    xmatrix.verify_request(&http_request, &destination, &public_key_map).unwrap();
}
