//! Types for the *m.dummy* event.

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

/// The payload for `DummyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.dummy")]
pub struct DummyToDeviceEventContent {}
