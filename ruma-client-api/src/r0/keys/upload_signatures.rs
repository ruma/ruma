//! POST /_matrix/client/r0/keys/signatures/upload
//!
//! Defined in [MSC 1756](https://github.com/matrix-org/matrix-doc/blob/master/proposals/1756-cross-signing.md#uploading-signing-keys)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Publishes cross-signing signatures for the user.",
        method: POST,
        name: "upload_signatures",
        path: "/_matrix/client/r0/keys/signatures/upload",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// Signed keys.
        #[ruma_api(body)]
        pub signed_keys: BTreeMap<UserId, BTreeMap<String, serde_json::Value>>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given signed keys.
    pub fn new(signed_keys: BTreeMap<UserId, BTreeMap<String, serde_json::Value>>) -> Self {
        Self { signed_keys }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
