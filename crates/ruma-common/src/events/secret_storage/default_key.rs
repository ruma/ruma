//! Types for the *m.secret_storage.default_key* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::GlobalAccountDataEvent;

/// An event to mark a key as the "default" key in the user's account_data.
pub type DefaultKeyEvent = GlobalAccountDataEvent<DefaultKeyEventContent>;

/// The payload for `DefaultKeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret_storage.default_key", kind = GlobalAccountData)]
pub struct DefaultKeyEventContent {
    /// The ID of the default key.
    pub key: String,
}
