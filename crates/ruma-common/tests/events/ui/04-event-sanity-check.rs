// Required, probably until Rust 1.57
// https://github.com/rust-lang/rust/issues/55779
#[allow(unused_extern_crates)]
extern crate serde;

use ruma_common::{
    events::{StateEventContent, StateUnsigned},
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId,
};
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct OriginalStateEvent<C: StateEventContent> {
    pub content: C,
    pub event_id: OwnedEventId,
    pub sender: OwnedUserId,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: OwnedRoomId,
    pub state_key: C::StateKey,
    pub unsigned: StateUnsigned<C>,
}

fn main() {}
