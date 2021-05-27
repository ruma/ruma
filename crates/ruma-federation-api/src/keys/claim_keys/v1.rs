//! [POST
//! /_matrix/federation/v1/user/keys/claim](https://matrix.org/docs/spec/server_server/r0.1.4#post-matrix-federation-v1-user-keys-claim)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::encryption::OneTimeKey;
use ruma_identifiers::{DeviceIdBox, DeviceKeyAlgorithm, DeviceKeyId, UserId};
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        path: "/_matrix/federation/v1/user/keys/claim",
        rate_limited: false,
        authentication: ServerSignatures,
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
pub type OneTimeKeyClaims = BTreeMap<UserId, BTreeMap<DeviceIdBox, DeviceKeyAlgorithm>>;

/// One time keys for use in pre-key messages
pub type OneTimeKeys = BTreeMap<UserId, BTreeMap<DeviceIdBox, BTreeMap<DeviceKeyId, OneTimeKey>>>;

/// A key and its signature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct KeyObject {
    /// The key, encoded using unpadded base64.
    pub key: String,

    /// Signature of the key object.
    pub signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,
}

impl KeyObject {
    /// Creates a new `KeyObject` with the given key and signatures.
    pub fn new(key: String, signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>) -> Self {
        Self { key, signatures }
    }
}
