//! `Deserialize` implementation for MessageEventContent and MessageType.

use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    from_raw_json_value,
    room::message::{MessageEventContent, MessageType, Relation},
};

/// Helper struct to determine the msgtype, relates_to and new_content fields
/// from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
struct MessageContentDeHelper {
    #[serde(rename = "m.relates_to")]
    relates_to: Option<Relation>,

    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.new_content")]
    new_content: Option<Box<MessageEventContent>>,
}

impl<'de> Deserialize<'de> for MessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let helper = from_raw_json_value::<MessageContentDeHelper, D::Error>(&json)?;

        Ok(Self {
            msgtype: from_raw_json_value(&json)?,
            relates_to: helper.relates_to,
            #[cfg(feature = "unstable-pre-spec")]
            new_content: helper.new_content,
        })
    }
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
            #[cfg(feature = "unstable-pre-spec")]
            "m.key.verification.request" => Self::VerificationRequest(from_raw_json_value(&json)?),
            _ => Self::_Custom(from_raw_json_value(&json)?),
        })
    }
}
