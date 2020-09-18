//! [POST /_matrix/client/r0/keys/claim](https://matrix.org/docs/spec/client_server/r0.6.1#post-matrix-client-r0-keys-claim)

use std::collections::BTreeMap;

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceIdBox, DeviceKeyAlgorithm, DeviceKeyId, UserId};
use serde_json::Value as JsonValue;

use super::OneTimeKey;

ruma_api! {
    metadata: {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        path: "/_matrix/client/r0/keys/claim",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The time (in milliseconds) to wait when downloading keys from remote servers.
        /// 10 seconds is the recommended default.
        #[serde(
            with = "ruma_serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        pub timeout: Option<Duration>,

        /// The keys to be claimed.
        pub one_time_keys: BTreeMap<UserId, BTreeMap<DeviceIdBox, DeviceKeyAlgorithm>>,
    }

    response: {
        /// If any remote homeservers could not be reached, they are recorded here.
        /// The names of the properties are the names of the unreachable servers.
        pub failures: BTreeMap<String, JsonValue>,

        /// One-time keys for the queried devices.
        pub one_time_keys: BTreeMap<UserId, OneTimeKeys>,
    }

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given key claims and the recommended 10 second timeout.
    pub fn new(one_time_keys: BTreeMap<UserId, BTreeMap<DeviceIdBox, DeviceKeyAlgorithm>>) -> Self {
        Self { timeout: Some(Duration::from_secs(10)), one_time_keys }
    }
}

impl Response {
    /// Creates a new `Response` with the given keys and no failures.
    pub fn new(one_time_keys: BTreeMap<UserId, OneTimeKeys>) -> Self {
        Self { failures: BTreeMap::new(), one_time_keys }
    }
}

/// The one-time keys for a given device.
pub type OneTimeKeys = BTreeMap<DeviceIdBox, BTreeMap<DeviceKeyId, OneTimeKey>>;
