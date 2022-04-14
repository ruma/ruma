//! `POST /_matrix/federation/*/user/keys/claim`
//!
//! Endpoint to claim one-time keys for use in pre-key messages

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#post_matrixfederationv1userkeysclaim

    use std::collections::BTreeMap;

    use ruma_common::{
        api::ruma_api,
        encryption::OneTimeKey,
        serde::{Base64, Raw},
        DeviceKeyAlgorithm, OwnedDeviceId, OwnedDeviceKeyId, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Claims one-time keys for use in pre-key messages.",
            method: POST,
            name: "claim_keys",
            stable_path: "/_matrix/federation/v1/user/keys/claim",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        request: {
            /// The keys to be claimed.
            pub one_time_keys: OneTimeKeyClaims,
        }

        response: {
            /// One-time keys for the queried devices
            pub one_time_keys: OneTimeKeys,
        }
    }

    impl Request {
        /// Creates a new `Request` with the given one time key claims.
        pub fn new(one_time_keys: OneTimeKeyClaims) -> Self {
            Self { one_time_keys }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given one time keys.
        pub fn new(one_time_keys: OneTimeKeys) -> Self {
            Self { one_time_keys }
        }
    }

    /// A claim for one time keys
    pub type OneTimeKeyClaims = BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, DeviceKeyAlgorithm>>;

    /// One time keys for use in pre-key messages
    pub type OneTimeKeys =
        BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, BTreeMap<OwnedDeviceKeyId, Raw<OneTimeKey>>>>;

    /// A key and its signature
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct KeyObject {
        /// The key, encoded using unpadded base64.
        pub key: Base64,

        /// Signature of the key object.
        pub signatures: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceKeyId, String>>,
    }

    impl KeyObject {
        /// Creates a new `KeyObject` with the given key and signatures.
        pub fn new(
            key: Base64,
            signatures: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceKeyId, String>>,
        ) -> Self {
            Self { key, signatures }
        }
    }
}
