use ruma_common::{
    events::{StateEventContent, StateUnsigned},
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId,
};
use ruma_macros::Event;
use serde::de::DeserializeOwned;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct OriginalStateEvent<C: StateEventContent + DeserializeOwned> {
    pub content: C,
    pub event_id: OwnedEventId,
    pub sender: OwnedUserId,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: OwnedRoomId,
    pub state_key: C::StateKey,
    pub unsigned: StateUnsigned<C>,
}

fn main() {}
