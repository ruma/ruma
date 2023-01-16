use ruma_common::{
    events::StaticStateEventContent, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    OwnedUserId,
};
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct OriginalStateEvent<C: StaticStateEventContent> {
    pub content: C,
    pub event_id: OwnedEventId,
    pub sender: OwnedUserId,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: OwnedRoomId,
    pub state_key: C::StateKey,
    pub unsigned: C::Unsigned,
}

fn main() {}
