// Required, probably until Rust 1.57
// https://github.com/rust-lang/rust/issues/55779
#[allow(unused_extern_crates)]
extern crate serde;

use ruma_common::{
    events::{EventContent, StateEventType, Unsigned},
    EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId,
};
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: EventContent<EventType = StateEventType>> {
    pub content: C,
    pub event_id: Box<EventId>,
    pub sender: Box<UserId>,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: Box<RoomId>,
    pub state_key: String,
    pub prev_content: Option<C>,
    pub unsigned: Unsigned,
}

fn main() {}
