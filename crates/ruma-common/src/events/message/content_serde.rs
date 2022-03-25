//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use std::convert::TryFrom;

use serde::{ser::SerializeStruct, Deserialize, Serialize};
use thiserror::Error;

use super::{MessageContent, Text};

#[derive(Error, Debug)]
pub enum MessageContentSerdeError {
    #[error("missing field `m.message` or `m.text`")]
    MissingMessage,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct MessageContentSerDeHelper {
    /// Plain text short form, stable name.
    #[serde(rename = "m.text")]
    text_stable: Option<String>,

    /// Plain text short form, unstable name.
    #[serde(rename = "org.matrix.msc1767.text")]
    text_unstable: Option<String>,

    /// Long form, stable name.
    #[serde(rename = "m.message")]
    message_stable: Option<Vec<Text>>,

    /// Long form, unstable name.
    #[serde(rename = "org.matrix.msc1767.message")]
    message_unstable: Option<Vec<Text>>,
}

impl TryFrom<MessageContentSerDeHelper> for MessageContent {
    type Error = MessageContentSerdeError;

    fn try_from(helper: MessageContentSerDeHelper) -> Result<Self, Self::Error> {
        let MessageContentSerDeHelper {
            text_stable,
            text_unstable,
            message_stable,
            message_unstable,
        } = helper;

        if let Some(message) = message_stable.or(message_unstable) {
            Ok(Self(message))
        } else if let Some(text) = text_stable.or(text_unstable) {
            Ok(Self::plain(text))
        } else {
            Err(MessageContentSerdeError::MissingMessage)
        }
    }
}

impl Serialize for MessageContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut st = serializer.serialize_struct("MessageContent", 1)?;
        if self.len() == 1 && self[0].mimetype == "text/plain" {
            st.serialize_field("org.matrix.msc1767.text", &self[0].body)?;
        } else {
            st.serialize_field("org.matrix.msc1767.message", &self.0)?;
        }
        st.end()
    }
}
