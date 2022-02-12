//! [POST /_matrix/client/r0/keys/claim](https://matrix.org/docs/spec/client_server/r0.6.1#post-matrix-client-r0-keys-claim)

use std::{collections::BTreeMap, time::Duration};

use ruma_api::ruma_api;
use ruma_common::encryption::OneTimeKey;
use ruma_identifiers::{DeviceId, DeviceKeyAlgorithm, DeviceKeyId, UserId};
use ruma_serde::Raw;
use serde_json::Value as JsonValue;

ruma_api! {
    metadata: {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        r0: "/_matrix/client/r0/keys/claim",
        stable: "/_matrix/client/v3/keys/claim",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.0,
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
        pub one_time_keys: BTreeMap<Box<UserId>, BTreeMap<Box<DeviceId>, DeviceKeyAlgorithm>>,
    }

    response: {
        /// If any remote homeservers could not be reached, they are recorded here.
        /// The names of the properties are the names of the unreachable servers.
        pub failures: BTreeMap<String, JsonValue>,

        /// One-time keys for the queried devices.
        pub one_time_keys: BTreeMap<Box<UserId>, OneTimeKeys>,
    }

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given key claims and the recommended 10 second timeout.
    pub fn new(
        one_time_keys: BTreeMap<Box<UserId>, BTreeMap<Box<DeviceId>, DeviceKeyAlgorithm>>,
    ) -> Self {
        Self { timeout: Some(Duration::from_secs(10)), one_time_keys }
    }
}

impl Response {
    /// Creates a new `Response` with the given keys and no failures.
    pub fn new(one_time_keys: BTreeMap<Box<UserId>, OneTimeKeys>) -> Self {
        Self { failures: BTreeMap::new(), one_time_keys }
    }
}

/// The one-time keys for a given device.
pub type OneTimeKeys = BTreeMap<Box<DeviceId>, BTreeMap<Box<DeviceKeyId>, Raw<OneTimeKey>>>;
