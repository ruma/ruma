// //! `Deserialize` implementation for MessageEventContent

use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    from_raw_json_value,
    room::message::{MessageEventContent, MessageType},
};

/// Helper struct to determine the msgtype, relates_to and new_content fields
/// from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
struct MessageContentDeHelper {
    msgtype: String,

    #[serde(rename = "m.relates_to")]
    relates_to: Option<Box<RawJsonValue>>,

    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.new_content")]
    new_content: Option<Box<RawJsonValue>>,
}

impl<'de> de::Deserialize<'de> for MessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        #[cfg(feature = "unstable-pre-spec")]
        let MessageContentDeHelper { msgtype, relates_to, new_content } =
            from_raw_json_value(&json)?;
        #[cfg(not(feature = "unstable-pre-spec"))]
        let MessageContentDeHelper { msgtype, relates_to } = from_raw_json_value(&json)?;

        Ok(match msgtype.as_str() {
            "m.audio" => Self {
                msgtype: MessageType::Audio(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.emote" => Self {
                msgtype: MessageType::Emote(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.file" => Self {
                msgtype: MessageType::File(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.image" => Self {
                msgtype: MessageType::Image(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.location" => Self {
                msgtype: MessageType::Location(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.notice" => Self {
                msgtype: MessageType::Notice(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.server_notice" => Self {
                msgtype: MessageType::ServerNotice(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.text" => Self {
                msgtype: MessageType::Text(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            "m.video" => Self {
                msgtype: MessageType::Video(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            #[cfg(feature = "unstable-pre-spec")]
            "m.key.verification.request" => Self {
                msgtype: MessageType::VerificationRequest(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
            _ => Self {
                msgtype: MessageType::_Custom(from_raw_json_value(&json)?),
                relates_to: relates_to.map(|json| from_raw_json_value(&json)).transpose()?,
                #[cfg(feature = "unstable-pre-spec")]
                new_content: new_content.map(|json| from_raw_json_value(&json)).transpose()?,
            },
        })
    }
}

/// Helper struct to determine the msgtype from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
struct MessageTypeDeHelper {
    /// The message type field
    msgtype: String,
}

impl<'de> de::Deserialize<'de> for MessageType {
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
