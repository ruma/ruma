//! [POST /_matrix/client/r0/keys/upload](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-keys-upload)

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::encryption::DeviceKeys;
use ruma_identifiers::{DeviceKeyAlgorithm, DeviceKeyId};

use super::OneTimeKey;

ruma_api! {
    metadata: {
        description: "Publishes end-to-end encryption keys for the device.",
        method: POST,
        name: "upload_keys",
        path: "/_matrix/client/r0/keys/upload",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// Identity keys for the device. May be absent if no new identity keys are required.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_keys: Option<DeviceKeys>,

        /// One-time public keys for "pre-key" messages.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub one_time_keys: Option<BTreeMap<DeviceKeyId, OneTimeKey>>,
    }

    response: {
        /// For each key algorithm, the number of unclaimed one-time keys of that
        /// type currently held on the server for this device.
        pub one_time_key_counts: BTreeMap<DeviceKeyAlgorithm, UInt>
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given one time key counts.
    pub fn new(one_time_key_counts: BTreeMap<DeviceKeyAlgorithm, UInt>) -> Self {
        Self { one_time_key_counts }
    }
}
