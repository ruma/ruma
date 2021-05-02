//! [GET /_matrix/identity/v2/pubkey/{keyId}](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2-pubkey-keyid)

use ruma_api::ruma_api;
use ruma_identifiers::ServerSigningKeyId;

ruma_api! {
    metadata: {
        description: "Get the public key for the given key ID.",
        method: GET,
        name: "get_public_key",
        path: "/_matrix/identity/v2/pubkey/:key_id",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The ID of the key.
        #[ruma_api(path)]
        pub key_id: ServerSigningKeyId,
    }

    response: {
        /// Unpadded Base64 encoded public key.
        pub public_key: String,
    }
}

impl Request {
    /// Create a `Request` with the given key_id.
    pub fn new(key_id: ServerSigningKeyId) -> Self {
        Self { key_id }
    }
}

impl Response {
    /// Create a `Response` with the given base64-encoded (unpadded) public key.
    pub fn new(public_key: String) -> Self {
        Self { public_key }
    }
}
