//! Types for the [`m.secret_storage.default_key`] event.
//!
//! [`m.secret_storage.default_key`]: https://spec.matrix.org/latest/client-server-api/#key-storage

use serde::{Deserialize, Serialize};

use crate::macros::EventContent;

/// The payload for `DefaultKeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret_storage.default_key", kind = GlobalAccountData)]
pub struct SecretStorageDefaultKeyEventContent {
    /// The ID of the default key.
    #[serde(rename = "key")]
    pub key_id: String,
}
