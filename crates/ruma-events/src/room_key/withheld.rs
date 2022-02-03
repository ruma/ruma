//! Types for the [`m.room_key.withheld`] event.
//!
//! [`m.room_key.withheld`]: https://spec.matrix.org/v1.1/client-server-api/#mroom_keywithheld

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.room_key.withheld` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room_key", kind = ToDevice)]
pub struct ToDeviceRoomKeyWithheldEventContent {}
