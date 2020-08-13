//! [POST /_matrix/client/r0/keys/device_signing/upload](https://13301-24998719-gh.circle-artifacts.com/0/scripts/gen/client_server/unstable.html#post-matrix-client-r0-keys-device-signing-upload)

use ruma_api::ruma_api;

use super::CrossSigningKey;
use crate::r0::uiaa::{AuthData, IncomingAuthData};

ruma_api! {
    metadata: {
        description: "Publishes cross signing keys for the user.",
        method: POST,
        name: "upload_signing_keys",
        path: "/_matrix/client/r0/keys/device_signing/upload",
        rate_limited: false,
        requires_authentication: true,
    }

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

    response: {}

    error: crate::Error
}
