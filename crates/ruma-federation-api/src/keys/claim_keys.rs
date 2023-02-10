//! `POST /_matrix/federation/*/user/keys/claim`
//!
//! Claim one-time keys for use in pre-key messages.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#post_matrixfederationv1userkeysclaim

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        encryption::OneTimeKey,
        metadata,
        serde::{Base64, Raw},
        DeviceKeyAlgorithm, OwnedDeviceId, OwnedDeviceKeyId, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/user/keys/claim",
        }
    };

    /// Request type for the `claim_keys` endpoint.
    #[request]
    pub struct Request {
        /// The keys to be claimed.
        pub one_time_keys: OneTimeKeyClaims,
    }

    /// Response type for the `claim_keys` endpoint.
    #[response]
    pub struct Response {
        /// One-time keys for the queried devices
        pub one_time_keys: OneTimeKeys,
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
