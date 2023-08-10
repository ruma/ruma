use serde::{de, Deserialize, Deserializer};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    OriginalRoomRedactionEvent, OriginalSyncRoomRedactionEvent, RoomRedactionEvent,
    SyncRoomRedactionEvent,
};
use crate::{events::RedactionDeHelper, serde::from_raw_json_value};

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
struct RoomRedactionDeHelper {
    content: RoomRedactionContentDeHelper,
    redacts: Option<de::IgnoredAny>,
}

#[derive(Deserialize)]
struct RoomRedactionContentDeHelper {
    redacts: Option<de::IgnoredAny>,
}

impl<'de> Deserialize<'de> for OriginalRoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RoomRedactionDeHelper { content, redacts } = from_raw_json_value(&json)?;

        match (redacts, content.redacts) {
            (Some(_), Some(_)) => Ok(Self::V1V11Compat(from_raw_json_value(&json)?)),
            (Some(_), None) => Ok(Self::V1(from_raw_json_value(&json)?)),
            (None, Some(_)) => Ok(Self::V11(from_raw_json_value(&json)?)),
            (None, None) => Err(de::Error::missing_field("redacts")),
        }
    }
}

impl<'de> Deserialize<'de> for OriginalSyncRoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RoomRedactionDeHelper { content, redacts } = from_raw_json_value(&json)?;

        match (redacts, content.redacts) {
            (Some(_), Some(_)) => Ok(Self::V1V11Compat(from_raw_json_value(&json)?)),
            (Some(_), None) => Ok(Self::V1(from_raw_json_value(&json)?)),
            (None, Some(_)) => Ok(Self::V11(from_raw_json_value(&json)?)),
            (None, None) => Err(de::Error::missing_field("redacts")),
        }
    }
}
