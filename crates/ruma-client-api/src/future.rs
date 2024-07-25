//! Endpoints for sending and receiving futures

pub mod send_future_message_event;
pub mod send_future_state_event;
pub mod update_future;

use serde::{Deserialize, Serialize};
use web_time::Duration;

/// The query parameters for a future request.
/// It can contain the possible `timeout` and `future_parent_id` combinations.
///
/// This is an Enum since the following properties might be added:
///
/// The **Timeout** case might get an additional `future_parent_id` property.
/// The optional parent id is used to create a secondary timeout.  
/// In a future group with two timeouts only one of them will ever be sent.
///
/// The **Action** case might be added:
/// Adds an additional action to a future without a timeout but requires a future group_id.
/// A possible matrix event that this future group can resolve to. It can be sent by using the
/// send_token as an alternative to the timeout future of an already existing group.

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum FutureParameters {
    /// Only sending the `delay` creates a timeout future with a new (server generated)
    /// future id (delay_id).
    Timeout {
        /// The timeout duration for this Future.
        #[serde(with = "ruma_common::serde::duration::ms")]
        #[serde(rename = "org.matrix.msc4140.delay")]
        timeout: Duration,
    },
}
