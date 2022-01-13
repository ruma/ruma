//! Server discovery endpoints.

use std::collections::BTreeMap;

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{ServerName, ServerSigningKeyId};
use ruma_serde::Base64;
use serde::{Deserialize, Serialize};

pub mod discover_homeserver;
pub mod get_remote_server_keys;
pub mod get_remote_server_keys_batch;
pub mod get_server_keys;
pub mod get_server_version;

/// Public key of the homeserver for verifying digital signatures.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VerifyKey {
    /// The unpadded base64-encoded key.
    pub key: Base64,
}

impl VerifyKey {
    /// Creates a new `VerifyKey` from the given key.
    pub fn new(key: Base64) -> Self {
        Self { key }
    }
}

/// A key the server used to use, but stopped using.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct OldVerifyKey {
    /// Timestamp when this key expired.
    pub expired_ts: MilliSecondsSinceUnixEpoch,

    /// The unpadded base64-encoded key.
    pub key: Base64,
}

impl OldVerifyKey {
    /// Creates a new `OldVerifyKey` with the given expiry time and key.
    pub fn new(expired_ts: MilliSecondsSinceUnixEpoch, key: Base64) -> Self {
        Self { expired_ts, key }
    }
}

// Spec is wrong, all fields are required (see https://github.com/matrix-org/matrix-doc/issues/2508)
/// Queried server key, signed by the notary server.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ServerSigningKeys {
    /// DNS name of the homeserver.
    pub server_name: Box<ServerName>,

    /// Public keys of the homeserver for verifying digital signatures.
    pub verify_keys: BTreeMap<Box<ServerSigningKeyId>, VerifyKey>,

    /// Public keys that the homeserver used to use and when it stopped using them.
    pub old_verify_keys: BTreeMap<Box<ServerSigningKeyId>, OldVerifyKey>,

    /// Digital signatures of this object signed using the verify_keys.
    ///
    /// Map of server name to keys by key ID.
    pub signatures: BTreeMap<Box<ServerName>, BTreeMap<Box<ServerSigningKeyId>, String>>,

    /// Timestamp when the keys should be refreshed.
    ///
    /// This field MUST be ignored in room versions 1, 2, 3, and 4.
    pub valid_until_ts: MilliSecondsSinceUnixEpoch,
}

impl ServerSigningKeys {
    /// Creates a new `ServerSigningKeys` with the given server name and validity timestamp.
    ///
    /// All other fields will be empty.
    pub fn new(server_name: Box<ServerName>, valid_until_ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self {
            server_name,
            verify_keys: BTreeMap::new(),
            old_verify_keys: BTreeMap::new(),
            signatures: BTreeMap::new(),
            valid_until_ts,
        }
    }
}
