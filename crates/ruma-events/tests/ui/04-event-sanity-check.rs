// Required, probably until Rust 1.57
// https://github.com/rust-lang/rust/issues/55779
#[allow(unused_extern_crates)]
extern crate serde;

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{StateEventContent, Unsigned};
use ruma_macros::Event;
use ruma_identifiers::{EventId, RoomId, UserId};

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent> {
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
