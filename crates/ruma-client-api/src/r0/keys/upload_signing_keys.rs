//! POST /_matrix/client/r0/keys/device_signing/upload
//!
//! Defined in [MSC 1756](https://github.com/matrix-org/matrix-doc/blob/master/proposals/1756-cross-signing.md#uploading-signing-keys)

use ruma_api::ruma_api;

use super::CrossSigningKey;
use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Publishes cross signing keys for the user.",
        method: POST,
        name: "upload_signing_keys",
        path: "/_matrix/client/unstable/keys/device_signing/upload",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,

        /// The user's master key.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub master_key: Option<CrossSigningKey>,

        /// The user's self-signing key. Must be signed with the accompanied master, or by the
        /// user's most recently uploaded master key if no master key is included in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub self_signing_key: Option<CrossSigningKey>,

        /// The user's user-signing key. Must be signed with the accompanied master, or by the
        /// user's most recently uploaded master key if no master key is included in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_signing_key: Option<CrossSigningKey>,
    }

    #[derive(Default)]
    response: {}

    error: UiaaResponse
}

impl Request<'_> {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
