//! Types for *m.notifications_profile* events.
//!
//! See [MSC2785](https://github.com/matrix-org/matrix-doc/pull/2785) for more details.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::GlobalAccountDataEvent;

pub mod some_profile;

/// An event to set a "notifications profile" in a user's `account_data`.
pub type NotificationsProfileEvent = GlobalAccountDataEvent<NotificationsProfileEventContent>;

/// The payload for `NotificationsProfileEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc2785.notifications_profile", kind = GlobalAccountData)]
pub struct NotificationsProfileEventContent {
    /// The user's "notifications profile".
    pub profile: String,
}

impl NotificationsProfileEventContent {
    /// Creates a new `NotificationsProfileEventContent` with the given profile name.
    pub fn new(profile: String) -> Self {
        Self { profile }
    }
}
