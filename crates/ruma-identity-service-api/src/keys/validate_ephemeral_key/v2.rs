//! [GET /_matrix/identity/v2/pubkey/ephemeral/isvalid](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2-pubkey-ephemeral-isvalid)

use ruma_api::ruma_api;
use ruma_serde::Base64;

ruma_api! {
    metadata: {
        description: "Check whether a short-term public key is valid.",
        method: GET,
        name: "validate_ephemeral_key",
        stable_path: "/_matrix/identity/v2/pubkey/ephemeral/isvalid",
        authentication: None,
        rate_limited: false,
        added: 1.0,
    }

    request: {
        /// The unpadded base64-encoded short-term public key to check.
        #[ruma_api(query)]
        pub public_key: &'a Base64,
    }

    response: {
        /// Whether the short-term public key is recognised and is currently valid.
        pub valid: bool,
    }
}

impl<'a> Request<'a> {
    /// Create a `Request` with the given base64-encoded (unpadded) short-term public key.
    pub fn new(public_key: &'a Base64) -> Self {
        Self { public_key }
    }
}

impl Response {
    /// Create a `Response` with the given bool indicating the validity of the short-term public
    /// key.
    pub fn new(valid: bool) -> Self {
        Self { valid }
    }
}
