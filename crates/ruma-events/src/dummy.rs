//! Types for the *m.dummy* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The payload for `DummyEvent`.
///
/// This event type is used to indicate new Olm sessions for end-to-end encryption.
///
/// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
///
/// The event does not have any content associated with it. The sending client is expected to
/// send a key share request shortly after this message, causing the receiving client to process
/// this *m.dummy* event as the most recent event and using the keyshare request to set up the
/// session. The keyshare request and *m.dummy* combination should result in the original
/// sending client receiving keys over the newly established session.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.dummy")]
pub struct DummyToDeviceEventContent {}
