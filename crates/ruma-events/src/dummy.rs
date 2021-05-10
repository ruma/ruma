//! Types for the *m.dummy* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The payload for `DummyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.dummy")]
pub struct DummyToDeviceEventContent {}
