use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId};
use ruma_events::StaticStateEventContent;
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StaticStateEventContent> {
    pub content: C,
    pub event_id: OwnedEventId,
    pub sender: OwnedUserId,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: OwnedRoomId,
    pub state_key: C::StateKey,
    pub unsigned: C::Unsigned,
    #[ruma_event(default, default_on_error)]
    pub custom_flag: bool,
    #[ruma_event(rename = "unstable_name", alias = "stable_name")]
    pub renamed_field: String,
}

fn main() {}
