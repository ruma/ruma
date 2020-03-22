//! [POST /_matrix/client/r0/keys/claim](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-keys-claim)

use std::collections::HashMap;

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceId, UserId};
use serde_json::Value;

use super::{AlgorithmAndDeviceId, KeyAlgorithm, OneTimeKey};

ruma_api! {
    metadata {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        path: "/_matrix/client/r0/keys/claim",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The time (in milliseconds) to wait when downloading keys from remote servers.
        /// 10 seconds is the recommended default.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::serde::duration::opt_ms")]
        pub timeout: Option<Duration>,

        /// The keys to be claimed.
        pub one_time_keys: HashMap<UserId, HashMap<DeviceId, KeyAlgorithm>>,
    }

    response {
        /// If any remote homeservers could not be reached, they are recorded here.
        /// The names of the properties are the names of the unreachable servers.
        pub failures: HashMap<String, Value>,

        /// One-time keys for the queried devices.
        pub one_time_keys: HashMap<UserId, HashMap<DeviceId, HashMap<AlgorithmAndDeviceId, OneTimeKey>>>,
    }

    error: crate::Error
}
