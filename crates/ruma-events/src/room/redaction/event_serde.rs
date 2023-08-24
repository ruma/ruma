use ruma_common::{
    serde::from_raw_json_value, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId,
};
use serde::{de, Deserialize, Deserializer};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    OriginalRoomRedactionEvent, OriginalSyncRoomRedactionEvent, RoomRedactionEvent,
    RoomRedactionEventContent, RoomRedactionUnsigned, SyncRoomRedactionEvent,
};
use crate::RedactionDeHelper;

impl<'de> Deserialize<'de> for RoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RedactionDeHelper { unsigned } = from_raw_json_value(&json)?;

        if unsigned.and_then(|u| u.redacted_because).is_some() {
            Ok(Self::Redacted(from_raw_json_value(&json)?))
        } else {
            Ok(Self::Original(from_raw_json_value(&json)?))
        }
    }
}

impl<'de> Deserialize<'de> for SyncRoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RedactionDeHelper { unsigned } = from_raw_json_value(&json)?;

        if unsigned.and_then(|u| u.redacted_because).is_some() {
            Ok(Self::Redacted(from_raw_json_value(&json)?))
        } else {
            Ok(Self::Original(from_raw_json_value(&json)?))
        }
    }
}

#[derive(Deserialize)]
struct OriginalRoomRedactionEventDeHelper {
    content: RoomRedactionEventContent,
    redacts: Option<OwnedEventId>,
    event_id: OwnedEventId,
    sender: OwnedUserId,
    origin_server_ts: MilliSecondsSinceUnixEpoch,
    room_id: Option<OwnedRoomId>,
    #[serde(default)]
    unsigned: RoomRedactionUnsigned,
}

impl<'de> Deserialize<'de> for OriginalRoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let OriginalRoomRedactionEventDeHelper {
            content,
            redacts,
            event_id,
            sender,
            origin_server_ts,
            room_id,
            unsigned,
        } = from_raw_json_value(&json)?;

        let Some(room_id) = room_id else { return Err(de::Error::missing_field("room_id")) };

        if redacts.is_none() && content.redacts.is_none() {
            return Err(de::Error::missing_field("redacts"));
        }

        Ok(Self { content, redacts, event_id, sender, origin_server_ts, room_id, unsigned })
    }
}

impl<'de> Deserialize<'de> for OriginalSyncRoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let OriginalRoomRedactionEventDeHelper {
            content,
            redacts,
            event_id,
            sender,
            origin_server_ts,
            unsigned,
            ..
        } = from_raw_json_value(&json)?;

        if redacts.is_none() && content.redacts.is_none() {
            return Err(de::Error::missing_field("redacts"));
        }

        Ok(Self { content, redacts, event_id, sender, origin_server_ts, unsigned })
    }
}
