//! `GET /_matrix/client/*/sync`

use js_int::UInt;
use serde::{self, Deserialize, Serialize};

pub mod v3;

#[cfg(feature = "unstable-msc3575")]
pub mod v4;

/// Unread notifications count.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnreadNotificationsCount {
    /// The number of unread notifications for this room with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<UInt>,

    /// The total number of unread notifications for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<UInt>,
}

impl UnreadNotificationsCount {
    /// Creates an empty `UnreadNotificationsCount`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no notification count updates.
    pub fn is_empty(&self) -> bool {
        self.highlight_count.is_none() && self.notification_count.is_none()
    }
}
