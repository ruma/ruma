//! Types for the [`m.fully_read`] object.
//!
//! [`m.fully_read`]: https://spec.matrix.org/v1.2/client-server-api/#mfully_read

use ruma_identifiers::EventId;
use ruma_macros::AccountDataContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.fully_read` object.
///
/// The current location of the user's read marker in a room.
///
/// This object appears in the user's room account data for the room the marker is applicable for.
#[derive(Clone, Debug, Deserialize, Serialize, AccountDataContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[account_data(type = "m.fully_read", kind = Room)]
pub struct FullyReadContent {
    /// The event the user's read marker is located at in the room.
    pub event_id: Box<EventId>,
}

impl FullyReadContent {
    /// Creates a new `FullyReadContent` with the given event ID.
    pub fn new(event_id: Box<EventId>) -> Self {
        Self { event_id }
    }
}
