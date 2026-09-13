use ruma_common::{EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId};
use ruma_events::StaticStateEventContent;
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct OriginalStateEvent<C: StaticStateEventContent> {
    pub content: C,
    pub event_id: EventId,
    pub sender: UserId,
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    pub room_id: RoomId,
    pub state_key: C::StateKey,
    pub unsigned: C::Unsigned,
    #[ruma_event(default, default_on_error)]
    pub custom_flag: bool,
    #[ruma_event(rename = "unstable_name", alias = "stable_name")]
    pub renamed_field: String,
}

fn main() {}
