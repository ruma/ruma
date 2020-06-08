//! Server discovery endpoints.

use std::{collections::BTreeMap, time::SystemTime};

use ruma_identifiers::ServerKeyId;
use serde::{Deserialize, Serialize};

pub mod discover_homeserver;
pub mod get_remote_server_keys;
pub mod get_remote_server_keys_batch;
pub mod get_server_keys;
pub mod get_server_version;

/// Public key of the homeserver for verifying digital signatures.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerifyKey {
    /// The Unpadded Base64 encoded key.
    pub key: String,
}

/// A key the server used to use, but stopped using.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OldVerifyKey {
    /// Timestamp when this key expired.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub expired_ts: SystemTime,
    /// The Unpadded Base64 encoded key.
    pub key: String,
}

// Spec is wrong, all fields are required (see
// https://github.com/matrix-org/matrix-doc/issues/2508)
/// Queried server key, signed by the notary server.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerKey {
    /// DNS name of the homeserver.
    pub server_name: String,
    /// Public keys of the homeserver for verifying digital signatures.
    pub verify_keys: BTreeMap<String, VerifyKey>,
    /// Public keys that the homeserver used to use and when it stopped using them.
    pub old_verify_keys: BTreeMap<String, OldVerifyKey>,
    /// Digital signatures of this object signed using the verify_keys. Map of
    /// server name to keys by key ID
    pub signatures: BTreeMap<String, BTreeMap<ServerKeyId, String>>,
    /// Timestamp when the keys should be refreshed. This field MUST be ignored in room
    /// versions 1, 2, 3, and 4.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub valid_until_ts: SystemTime,
}
