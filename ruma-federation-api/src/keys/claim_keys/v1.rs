//! [POST
//! /_matrix/federation/v1/user/keys/claim](https://matrix.org/docs/spec/server_server/r0.1.4#post-matrix-federation-v1-user-keys-claim)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceIdBox, DeviceKeyAlgorithm, DeviceKeyId, UserId};
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        path: "/_matrix/federation/v1/user/keys/claim",
        rate_limited: false,
        requires_authentication: true,
    }

    #[non_exhaustive]
    request: {
        /// The keys to be claimed.
        one_time_keys: OneTimeKeyClaims,
    }

    #[non_exhaustive]
    response: {
        /// One-time keys for the queried devices
        one_time_keys: OneTimeKeys,
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
pub type OneTimeKeys = BTreeMap<UserId, BTreeMap<DeviceIdBox, BTreeMap<DeviceKeyId, KeyObject>>>;

/// A key and its signature
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyObject {
    /// The key, encoded using unpadded base64.
    key: String,
    /// Signature of the key object.
    signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,
}
