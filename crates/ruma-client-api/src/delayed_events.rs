//! Endpoints for sending and interacting with delayed events.

pub mod delayed_message_event;
pub mod delayed_state_event;
pub mod update_delayed_event;

use serde::{Deserialize, Serialize};
use web_time::Duration;

/// The query parameters for a delayed event request.
/// It contains the `timeout` configuration for a delayed event.
///
/// ### Note:
///
/// This is an Enum since the following properties might be added:
///
/// The **Timeout** case might get an additional optional `delay_parent_id` property.
/// The optional parent id is used to create a secondary timeout.  
/// In a delay group with two timeouts only one of them will ever be sent.
///
/// The **Action** case might be added:
/// Adds an additional action to a delay event without a timeout but requires a `delay_id` (of the
/// parent delay event). A possible matrix event that can be send as an alternative to the parent
/// delay.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum DelayParameters {
    /// Sending a delayed event with a timeout. The response will contain a (server
    /// generated) `delay_id` instead of an `event_id`.
    Timeout {
        /// The timeout duration for this delayed event.
        #[serde(with = "ruma_common::serde::duration::ms")]
        #[serde(rename = "org.matrix.msc4140.delay")]
        timeout: Duration,
    },
}
