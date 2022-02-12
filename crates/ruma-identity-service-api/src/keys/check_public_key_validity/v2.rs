//! [GET /_matrix/identity/v2/pubkey/isvalid](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2-pubkey-isvalid)

use ruma_api::ruma_api;
use ruma_serde::Base64;

ruma_api! {
    metadata: {
        description: "Check whether a long-term public key is valid. The response should always be the same, provided the key exists.",
        method: GET,
        name: "check_public_key_validity",
        stable: "/_matrix/identity/v2/pubkey/isvalid",
        authentication: None,
        rate_limited: false,
        added: 1.0,
    }

    request: {
        /// Base64-encoded (no padding) public key to check for validity.
        #[ruma_api(query)]
        pub public_key: &'a Base64,
    }

    response: {
        /// Whether the public key is recognised and is currently valid.
        pub valid: bool,
    }
}

impl<'a> Request<'a> {
    /// Create a `Request` with the given base64-encoded (unpadded) public key.
    pub fn new(public_key: &'a Base64) -> Self {
        Self { public_key }
    }
}

impl Response {
    /// Create a `Response` with the given bool indicating the validity of the public key.
    pub fn new(valid: bool) -> Self {
        Self { valid }
    }
}
