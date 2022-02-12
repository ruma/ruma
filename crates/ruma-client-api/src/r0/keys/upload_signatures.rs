//! [POST /_matrix/client/v3/keys/signatures/upload](https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3keyssignaturesupload)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde_json::Value as JsonValue;

ruma_api! {
    metadata: {
        description: "Publishes cross-signing signatures for the user.",
        method: POST,
        name: "upload_signatures",
        unstable_path: "/_matrix/client/unstable/keys/signatures/upload",
        stable_path: "/_matrix/client/v3/keys/signatures/upload",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.1,
    }

    request: {
        /// Signed keys.
        #[ruma_api(body)]
        pub signed_keys: BTreeMap<Box<UserId>, BTreeMap<String, JsonValue>>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given signed keys.
    pub fn new(signed_keys: BTreeMap<Box<UserId>, BTreeMap<String, JsonValue>>) -> Self {
        Self { signed_keys }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}
