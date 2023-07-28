//! `Deserialize` implementation for RoomMessageEventContent and MessageType.

use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    relation_serde::deserialize_relation, MessageType, RoomMessageEventContent,
    RoomMessageEventContentWithoutRelation,
};
use crate::{events::Mentions, serde::from_raw_json_value};

impl<'de> Deserialize<'de> for RoomMessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let mut deserializer = serde_json::Deserializer::from_str(json.get());
        let relates_to = deserialize_relation(&mut deserializer).map_err(de::Error::custom)?;

        let MentionsDeHelper { mentions } = from_raw_json_value(&json)?;

        Ok(Self { msgtype: from_raw_json_value(&json)?, relates_to, mentions })
    }
}

impl<'de> Deserialize<'de> for RoomMessageEventContentWithoutRelation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let MentionsDeHelper { mentions } = from_raw_json_value(&json)?;

        Ok(Self { msgtype: from_raw_json_value(&json)?, mentions })
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

#[allow(unreachable_pub)] // https://github.com/rust-lang/rust/issues/112615
#[cfg(feature = "unstable-msc3488")]
pub(in super::super) mod msc3488 {
    use serde::{Deserialize, Serialize};

    use crate::{
        events::{
            location::{AssetContent, LocationContent},
            message::historical_serde::MessageContentBlock,
            room::message::{LocationInfo, LocationMessageEventContent},
        },
        MilliSecondsSinceUnixEpoch,
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
