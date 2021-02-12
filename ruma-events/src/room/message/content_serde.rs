//! `Deserialize` implementation for MessageEventContent
use std::collections::BTreeMap;

use serde::{de, Deserialize};
use serde_json::value::{RawValue as RawJsonValue, Value as JsonValue};

use crate::{
    from_raw_json_value,
    room::message::{CustomEventContent, MessageEventContent},
};

/// Helper struct to determine the msgtype from a `serde_json::value::RawValue`
#[doc(hidden)]
#[derive(Debug, Deserialize)]
pub struct MessageDeHelper {
    /// The message type field
    msgtype: String,

    /// Everything else in the json object
    #[serde(flatten)]
    remaining: BTreeMap<String, JsonValue>,
}

impl<'de> de::Deserialize<'de> for MessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use MessageEventContent::*;
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let MessageDeHelper { msgtype, remaining } = from_raw_json_value(&json)?;
        Ok(match msgtype.as_ref() {
            "m.audio" => Audio(from_raw_json_value(&json)?),
            "m.emote" => Emote(from_raw_json_value(&json)?),
            "m.file" => File(from_raw_json_value(&json)?),
            "m.image" => Image(from_raw_json_value(&json)?),
            "m.location" => Location(from_raw_json_value(&json)?),
            "m.notice" => Notice(from_raw_json_value(&json)?),
            "m.server_notice" => ServerNotice(from_raw_json_value(&json)?),
            "m.text" => Text(from_raw_json_value(&json)?),
            "m.video" => Video(from_raw_json_value(&json)?),
            #[cfg(feature = "unstable-pre-spec")]
            "m.key.verification.request" => VerificationRequest(from_raw_json_value(&json)?),
            s => {
                let remaining = remaining.into_iter().collect::<serde_json::Map<_, _>>();
                _Custom(CustomEventContent { msgtype: s.to_string(), data: remaining })
            }
        })
    }
}
