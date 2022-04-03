//! Types for the *m.secret_storage.default_key* event.

use ruma_common::events::macros::EventContent;
use serde::{Deserialize, Serialize};

/// The payload for `DefaultKeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret_storage.default_key", kind = GlobalAccountData)]
pub struct DefaultKeyEventContent {
    /// The ID of the default key.
    pub key: String,
}
