//! [GET /_matrix/key/v2/server](https://matrix.org/docs/spec/server_server/r0.1.3#get-matrix-key-v2-server-keyid)

use std::{collections::BTreeMap, time::SystemTime};

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Gets the homeserver's published signing keys.",
        method: GET,
        name: "get_server_keys",
        path: "/_matrix/key/v2/server",
        rate_limited: false,
        requires_authentication: false,
    }

    request {}

    response {
        // Spec is wrong, all fields are required (see
        // https://github.com/matrix-org/matrix-doc/issues/2508)

        /// DNS name of the homeserver.
        pub server_name: String,
        /// Public keys of the homeserver for verifying digital signatures.
        pub verify_keys: BTreeMap<String, VerifyKey>,
        /// Public keys that the homeserver used to use and when it stopped using them.
        pub old_verify_keys: BTreeMap<String, OldVerifyKey>,
        /// Digital signatures of this object signed using the verify_keys.
        pub signatures: BTreeMap<String, BTreeMap<String, String>>,
        /// Timestamp when the keys should be refreshed. This field MUST be ignored in room
        /// versions 1, 2, 3, and 4.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub valid_until_ts: SystemTime,
    }
}

/// Public key of the homeserver for verifying digital signatures.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerifyKey {
    /// The Unpadded Base64 encoded key.
    pub key: String,
}

/// A key the server used to use, but stopped using.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OldVerifyKey {
    /// Timestamp when this key expired.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub expired_ts: SystemTime,
    /// The Unpadded Base64 encoded key.
    pub key: String,
}
