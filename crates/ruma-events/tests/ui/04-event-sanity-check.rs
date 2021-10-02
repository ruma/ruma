// rustc overflows when compiling this see:
// https://github.com/rust-lang/rust/issues/55779
extern crate serde;

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{StateEventContent, Unsigned};
use ruma_events_macros::Event;
use ruma_identifiers::{EventId, RoomId, UserId};

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent> {
    pub content: C,
    pub event_id: EventId,
    pub sender: UserId,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: RoomId,
    pub state_key: String,
    pub prev_content: Option<C>,
    pub unsigned: Unsigned,
}

fn main() {}
