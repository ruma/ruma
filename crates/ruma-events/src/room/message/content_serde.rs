//! `Deserialize` implementation for RoomMessageEventContent and MessageType.

use std::collections::BTreeMap;

use ruma_common::serde::from_raw_json_value;
use serde::{de, Deserialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

#[cfg(feature = "unstable-msc4274")]
use super::gallery::GalleryItemType;
use super::{
    relation_serde::deserialize_relation, MessageType, RoomMessageEventContent,
    RoomMessageEventContentWithoutRelation,
};
use crate::Mentions;

impl<'de> Deserialize<'de> for RoomMessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let mut deserializer = serde_json::Deserializer::from_str(json.get());
        let relates_to = deserialize_relation(&mut deserializer).map_err(de::Error::custom)?;

        let MentionsDeHelper { mentions } = from_raw_json_value(&json)?;

        // Extract custom fields by parsing the JSON and removing known fields
        let mut additional_fields = BTreeMap::new();
        if let Ok(mut map) = serde_json::from_str::<serde_json::Map<String, JsonValue>>(json.get())
        {
            // Remove known fields
            map.remove("msgtype");
            map.remove("body");
            map.remove("format");
            map.remove("formatted_body");
            map.remove("m.relates_to");
            map.remove("m.mentions");
            map.remove("m.new_content");
            // Also remove message type-specific fields
            map.remove("info");
            map.remove("file");
            map.remove("filename");
            map.remove("url");
            map.remove("geo_uri");
            map.remove("key");
            map.remove("server_notice_type");
            map.remove("admin_contact");
            map.remove("limit_type");
            map.remove("to");
            map.remove("from_device");
            map.remove("methods");
            // MSC-specific fields that are handled elsewhere
            map.remove("org.matrix.msc1767.text");
            map.remove("org.matrix.msc1767.html");
            map.remove("org.matrix.msc1767.message");
            map.remove("org.matrix.msc3245.voice");
            map.remove("org.matrix.msc1767.audio");
            map.remove("org.matrix.msc1767.file");
            map.remove("org.matrix.msc1767.image");
            map.remove("org.matrix.msc1767.video");
            map.remove("org.matrix.msc3551.extensible_events");
            // Convert remaining fields to additional_fields
            for (k, v) in map {
                additional_fields.insert(k, v);
            }
        }

        Ok(Self { msgtype: from_raw_json_value(&json)?, relates_to, mentions, additional_fields })
    }
}

impl<'de> Deserialize<'de> for RoomMessageEventContentWithoutRelation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let MentionsDeHelper { mentions } = from_raw_json_value(&json)?;

        // Extract custom fields by parsing the JSON and removing known fields
        let mut additional_fields = BTreeMap::new();
        if let Ok(mut map) = serde_json::from_str::<serde_json::Map<String, JsonValue>>(json.get())
        {
            // Remove known fields
            map.remove("msgtype");
            map.remove("body");
            map.remove("format");
            map.remove("formatted_body");
            map.remove("m.mentions");
            // Also remove message type-specific fields
            map.remove("info");
            map.remove("file");
            map.remove("filename");
            map.remove("url");
            map.remove("geo_uri");
            map.remove("key");
            map.remove("server_notice_type");
            map.remove("admin_contact");
            map.remove("limit_type");
            map.remove("to");
            map.remove("from_device");
            map.remove("methods");
            // MSC-specific fields that are handled elsewhere
            map.remove("org.matrix.msc1767.text");
            map.remove("org.matrix.msc1767.html");
            map.remove("org.matrix.msc1767.message");
            map.remove("org.matrix.msc3245.voice");
            map.remove("org.matrix.msc1767.audio");
            map.remove("org.matrix.msc1767.file");
            map.remove("org.matrix.msc1767.image");
            map.remove("org.matrix.msc1767.video");
            map.remove("org.matrix.msc3551.extensible_events");
            // Convert remaining fields to additional_fields
            for (k, v) in map {
                additional_fields.insert(k, v);
            }
        }

        Ok(Self { msgtype: from_raw_json_value(&json)?, mentions, additional_fields })
    }
}

#[derive(Deserialize)]
struct MentionsDeHelper {
    #[serde(rename = "m.mentions")]
    mentions: Option<Mentions>,
}

/// Helper struct to determine the msgtype from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
struct MessageTypeDeHelper {
    /// The message type field
    msgtype: String,
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let MessageTypeDeHelper { msgtype } = from_raw_json_value(&json)?;

        Ok(match msgtype.as_ref() {
            "m.audio" => Self::Audio(from_raw_json_value(&json)?),
            "m.emote" => Self::Emote(from_raw_json_value(&json)?),
            "m.file" => Self::File(from_raw_json_value(&json)?),
            #[cfg(feature = "unstable-msc4274")]
            "dm.filament.gallery" => Self::Gallery(from_raw_json_value(&json)?),
            "m.image" => Self::Image(from_raw_json_value(&json)?),
            "m.location" => Self::Location(from_raw_json_value(&json)?),
            "m.notice" => Self::Notice(from_raw_json_value(&json)?),
            "m.server_notice" => Self::ServerNotice(from_raw_json_value(&json)?),
            "m.text" => Self::Text(from_raw_json_value(&json)?),
            "m.video" => Self::Video(from_raw_json_value(&json)?),
            "m.key.verification.request" => Self::VerificationRequest(from_raw_json_value(&json)?),
            _ => Self::_Custom(from_raw_json_value(&json)?),
        })
    }
}

/// Helper struct to determine the itemtype from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
#[cfg(feature = "unstable-msc4274")]
struct ItemTypeDeHelper {
    /// The item type field
    itemtype: String,
}

#[cfg(feature = "unstable-msc4274")]
impl<'de> Deserialize<'de> for GalleryItemType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let ItemTypeDeHelper { itemtype } = from_raw_json_value(&json)?;

        Ok(match itemtype.as_ref() {
            "m.audio" => Self::Audio(from_raw_json_value(&json)?),
            "m.file" => Self::File(from_raw_json_value(&json)?),
            "m.image" => Self::Image(from_raw_json_value(&json)?),
            "m.video" => Self::Video(from_raw_json_value(&json)?),
            _ => Self::_Custom(from_raw_json_value(&json)?),
        })
    }
}

#[allow(unreachable_pub)] // https://github.com/rust-lang/rust/issues/112615
#[cfg(feature = "unstable-msc3488")]
pub(in super::super) mod msc3488 {
    use ruma_common::MilliSecondsSinceUnixEpoch;
    use serde::{Deserialize, Serialize};

    use crate::{
        location::{AssetContent, LocationContent},
        message::historical_serde::MessageContentBlock,
        room::message::{LocationInfo, LocationMessageEventContent},
    };

    /// Deserialize helper type for `LocationMessageEventContent` with unstable fields from msc3488.
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "msgtype", rename = "m.location")]
    pub(in super::super) struct LocationMessageEventContentSerDeHelper {
        pub body: String,

        pub geo_uri: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub info: Option<Box<LocationInfo>>,

        #[serde(flatten)]
        pub message: Option<MessageContentBlock>,

        #[serde(rename = "org.matrix.msc3488.location", skip_serializing_if = "Option::is_none")]
        pub location: Option<LocationContent>,

        #[serde(rename = "org.matrix.msc3488.asset", skip_serializing_if = "Option::is_none")]
        pub asset: Option<AssetContent>,

        #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
        pub ts: Option<MilliSecondsSinceUnixEpoch>,
    }

    impl From<LocationMessageEventContent> for LocationMessageEventContentSerDeHelper {
        fn from(value: LocationMessageEventContent) -> Self {
            let LocationMessageEventContent { body, geo_uri, info, message, location, asset, ts } =
                value;

            Self { body, geo_uri, info, message: message.map(Into::into), location, asset, ts }
        }
    }

    impl From<LocationMessageEventContentSerDeHelper> for LocationMessageEventContent {
        fn from(value: LocationMessageEventContentSerDeHelper) -> Self {
            let LocationMessageEventContentSerDeHelper {
                body,
                geo_uri,
                info,
                message,
                location,
                asset,
                ts,
            } = value;

            LocationMessageEventContent {
                body,
                geo_uri,
                info,
                message: message.map(Into::into),
                location,
                asset,
                ts,
            }
        }
    }
}
