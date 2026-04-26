use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedUserId, serde::from_raw_json_value,
};
use serde::{Deserialize, de};
use serde_json::value::RawValue as RawJsonValue;

use super::{AnyRedactionEvent, CustomRedactionEvent};
use crate::EventTypeDeHelper;

impl<'de> Deserialize<'de> for AnyRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventTypeDeHelper { ev_type, .. } = from_raw_json_value(&json)?;

        match &*ev_type {
            "m.room.redaction" => from_raw_json_value(&json).map(Self::RoomRedaction),
            #[cfg(feature = "unstable-msc4293")]
            "m.room.member" => from_raw_json_value(&json).map(Self::RoomMember),
            _ => {
                let CustomRedactionEventDeHelper { event_type, event_id, sender, origin_server_ts } =
                    from_raw_json_value(&json)?;
                Ok(Self::_Custom(CustomRedactionEvent {
                    event_type,
                    event_id,
                    sender,
                    origin_server_ts,
                }))
            }
        }
    }
}

#[derive(Deserialize)]
struct CustomRedactionEventDeHelper {
    /// The type of the event
    #[serde(rename = "type")]
    event_type: Box<str>,

    /// The globally unique event identifier for the user who sent the event.
    event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    origin_server_ts: MilliSecondsSinceUnixEpoch,
}
