//! Endpoints for sending and interacting with delayed events.
//!
//! Delayed events are an unstable feature added by [MSC4140](https://github.com/matrix-org/matrix-spec-proposals/pull/4140)

pub mod get_all_delayed_events;
pub mod get_delayed_event;
pub mod send_delayed_event;
pub mod update_delayed_event;

// deprecated endpoints
pub mod delayed_message_event;
pub mod delayed_state_event;

use std::time::Duration;

use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    api::error::StandardErrorBody,
    serde::{Raw, StringEnum},
};
use ruma_events::{AnyTimelineEventContent, TimelineEventType};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The details of a delayed event, as returned by the unstable `get_delayed_event::msc4140` and
/// `get_all_delayed_events::msc4140` endpoints.
///
/// This matches a previous iteration of MSC4140. See [`DelayedEvent`] for the current version.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DelayedEventData {
    /// The ID of the delayed event.
    pub delay_id: String,

    /// The ID of the room that the delayed event was scheduled to be sent in.
    pub room_id: OwnedRoomId,

    /// The event type of the delayed event.
    #[serde(rename = "type")]
    pub event_type: TimelineEventType,

    /// The State Key if the event is a state event, nothing otherwise
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,

    /// The event content to send.
    ///
    /// This is the content that was submitted to the send endpoint, not the content of the final
    /// event
    pub content: Raw<AnyTimelineEventContent>,

    /// The duration that the server should wait before sending this event
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub delay: Duration,

    /// The timestamp when the delayed event was scheduled or last restarted.
    pub running_since: MilliSecondsSinceUnixEpoch,

    /// The error that prevented the delayed event from being sent.
    /// Present only for finalized events that were cancelled due to an error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StandardErrorBody>,

    /// The event_id this event got when it was sent.
    /// Present only for events that were sent successfully.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<OwnedEventId>,

    /// The timestamp when the event was finalized.
    /// Present only for events that were finalized (sent, failed to send, or cancelled).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "finalised_ts")]
    pub finalized_ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl DelayedEventData {
    /// Create a new delayed event data object with the given parameters
    pub fn new(
        delay_id: String,
        room_id: OwnedRoomId,
        event_type: TimelineEventType,
        state_key: Option<String>,
        content: Raw<AnyTimelineEventContent>,
        delay: Duration,
        running_since: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self {
            delay_id,
            room_id,
            event_type,
            state_key,
            delay,
            running_since,
            content,
            error: None,
            event_id: None,
            finalized_ts: None,
        }
    }

    /// Returns the status indicated by this delayed event data.
    pub fn status(&self) -> DelayedEventStatus {
        if self.finalized_ts.is_none() {
            DelayedEventStatus::Scheduled
        } else if self.event_id.is_some() {
            DelayedEventStatus::Send
        } else if self.error.is_some() {
            DelayedEventStatus::Error
        } else {
            DelayedEventStatus::Cancel
        }
    }
}

/// The status that a delayed event stored on the server can have.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum DelayedEventStatus {
    /// The event is currently scheduled to be submitted at a later date.
    /// It may be restarted, sent or cancelled via the management endpoint.
    Scheduled,

    /// The event has been sent successfully.
    Send,

    /// The event has been cancelled.
    Cancel,

    /// The event has encountered an error when trying to send.
    Error,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The details of a delayed event, as defined in the accepted version of [MSC4140].
///
/// [MSC4140]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DelayedEvent {
    /// The ID of the delayed event.
    pub delay_id: String,

    /// The ID of the room that the delayed event was scheduled to be sent in.
    pub room_id: OwnedRoomId,

    /// The event type of the delayed event.
    #[serde(rename = "type")]
    pub event_type: TimelineEventType,

    /// The state key of the delayed event if it is a state event, `None` otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,

    /// The content of the delayed event.
    ///
    /// This is the content that was submitted to the send endpoint, not the content of the final
    /// event.
    pub content: Raw<AnyTimelineEventContent>,

    /// The delay after the point of scheduling that the event is or was to be sent at.
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub delay_ms: Duration,

    /// The timestamp when the delayed event was scheduled or last restarted.
    pub delayed_since_ts: MilliSecondsSinceUnixEpoch,

    /// Details on how the delayed event was finalized.
    ///
    /// Present only for finalized delayed events.
    #[serde(rename = "finalised", skip_serializing_if = "Option::is_none")]
    pub finalized: Option<FinalizedDelayedEvent>,
}

impl DelayedEvent {
    /// Creates a new `DelayedEvent` for a scheduled delayed event with the given parameters.
    pub fn new(
        delay_id: String,
        room_id: OwnedRoomId,
        event_type: TimelineEventType,
        state_key: Option<String>,
        content: Raw<AnyTimelineEventContent>,
        delay_ms: Duration,
        delayed_since_ts: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self {
            delay_id,
            room_id,
            event_type,
            state_key,
            content,
            delay_ms,
            delayed_since_ts,
            finalized: None,
        }
    }

    /// Returns the status indicated by this delayed event.
    pub fn status(&self) -> DelayedEventStatus {
        match &self.finalized {
            None => DelayedEventStatus::Scheduled,
            Some(finalized) => finalized.status(),
        }
    }
}

/// Details on how a delayed event was finalized.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct FinalizedDelayedEvent {
    /// The error that prevented the delayed event from being sent.
    ///
    /// Present only for delayed events that were cancelled due to an error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StandardErrorBody>,

    /// The ID of the event that was sent.
    ///
    /// Present only for delayed events that were sent. Mutually exclusive with `error`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<OwnedEventId>,

    /// The timestamp when the delayed event was finalized.
    #[serde(rename = "finalised_ts")]
    pub finalized_ts: MilliSecondsSinceUnixEpoch,
}

impl FinalizedDelayedEvent {
    /// Creates a new `FinalizedDelayedEvent` for a delayed event that was sent with the given
    /// event ID.
    pub fn sent(event_id: OwnedEventId, finalized_ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { error: None, event_id: Some(event_id), finalized_ts }
    }

    /// Creates a new `FinalizedDelayedEvent` for a delayed event that was cancelled due to the
    /// given error.
    pub fn failed(error: StandardErrorBody, finalized_ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { error: Some(error), event_id: None, finalized_ts }
    }

    /// Creates a new `FinalizedDelayedEvent` for a delayed event that was cancelled.
    pub fn cancelled(finalized_ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { error: None, event_id: None, finalized_ts }
    }

    /// Returns the status indicated by this finalized delayed event.
    pub fn status(&self) -> DelayedEventStatus {
        if self.event_id.is_some() {
            DelayedEventStatus::Send
        } else if self.error.is_some() {
            DelayedEventStatus::Error
        } else {
            DelayedEventStatus::Cancel
        }
    }
}

impl From<DelayedEventData> for DelayedEvent {
    fn from(data: DelayedEventData) -> Self {
        let DelayedEventData {
            delay_id,
            room_id,
            event_type,
            state_key,
            content,
            delay,
            running_since,
            error,
            event_id,
            finalized_ts,
        } = data;

        Self {
            delay_id,
            room_id,
            event_type,
            state_key,
            content,
            delay_ms: delay,
            delayed_since_ts: running_since,
            finalized: finalized_ts.map(|finalized_ts| FinalizedDelayedEvent {
                error,
                event_id,
                finalized_ts,
            }),
        }
    }
}

impl From<DelayedEvent> for DelayedEventData {
    fn from(event: DelayedEvent) -> Self {
        let DelayedEvent {
            delay_id,
            room_id,
            event_type,
            state_key,
            content,
            delay_ms,
            delayed_since_ts,
            finalized,
        } = event;
        let (error, event_id, finalized_ts) = match finalized {
            Some(FinalizedDelayedEvent { error, event_id, finalized_ts }) => {
                (error, event_id, Some(finalized_ts))
            }
            None => (None, None, None),
        };

        Self {
            delay_id,
            room_id,
            event_type,
            state_key,
            content,
            delay: delay_ms,
            running_since: delayed_since_ts,
            error,
            event_id,
            finalized_ts,
        }
    }
}

/// The query parameters for a delayed event request.
/// It contains the `timeout` configuration for a delayed event.
///
/// This enum is no longer used except by deprecated endpoints.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
