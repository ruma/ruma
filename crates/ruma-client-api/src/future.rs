//! Endpoints for sending and receiving futures

pub mod future_message_event;
pub mod future_state_event;

pub mod send_future_update;

use ::serde::{Deserialize, Serialize};
use web_time::Duration;

/// The query parameters for a future request.
/// It can contain the possible timeout and the future_group_id combinations.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FutureParameters {
    /// Only sending the timeout creates a timeout future with a new (server generated)
    /// group id. The optoinal group id is used to create a secondary timeout.
    /// In a future group with two timeouts only one of them will ever be sent.
    Timeout {
        /// the timeout duration for this Future.
        #[serde(with = "ruma_common::serde::duration::ms")]
        #[serde(rename = "future_timeout")]
        timeout: Duration,
        /// The associated group for this Future.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "future_group_id")]
        group_id: Option<String>,
    },

    /// Adds an additional action to a future without a timeout but requires a future group_id
    /// A possible matrix event that this future group  can resolve to. It can be send by using the
    /// send_token as an alternative to the timeout future of an already existing group.
    Action {
        /// The associated group for this Future.
        group_id: String,
    },
}
