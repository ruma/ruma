//! `POST /_matrix/client/*/keys/upload`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3keysupload

    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        api::ruma_api,
        encryption::{DeviceKeys, OneTimeKey},
        serde::Raw,
        DeviceKeyAlgorithm, OwnedDeviceKeyId,
    };

    ruma_api! {
        metadata: {
            description: "Publishes end-to-end encryption keys for the device.",
            method: POST,
            name: "upload_keys",
            r0_path: "/_matrix/client/r0/keys/upload",
            stable_path: "/_matrix/client/v3/keys/upload",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {
            /// Identity keys for the device.
            ///
            /// May be absent if no new identity keys are required.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub device_keys: Option<Raw<DeviceKeys>>,

            /// One-time public keys for "pre-key" messages.
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            pub one_time_keys: BTreeMap<OwnedDeviceKeyId, Raw<OneTimeKey>>,

            /// Fallback public keys for "pre-key" messages.
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty", rename = "org.matrix.msc2732.fallback_keys")]
            pub fallback_keys: BTreeMap<OwnedDeviceKeyId, Raw<OneTimeKey>>,
        }

        response: {
            /// For each key algorithm, the number of unclaimed one-time keys of that
            /// type currently held on the server for this device.
            pub one_time_key_counts: BTreeMap<DeviceKeyAlgorithm, UInt>,
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
}
