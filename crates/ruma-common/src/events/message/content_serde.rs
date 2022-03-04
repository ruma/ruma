//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use std::convert::TryFrom;

use serde::{ser::SerializeStruct, Deserialize, Serialize};
use thiserror::Error;

use super::{MessageContent, Text};

#[derive(Error, Debug)]
pub enum MessageContentSerdeError {
    #[error("missing field `{0}`")]
    MissingField(String),
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct MessageContentSerDeHelper {
    /// Plain text short form.
    #[serde(rename = "org.matrix.msc1767.text", skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// Long form.
    #[serde(rename = "org.matrix.msc1767.message", default, skip_serializing_if = "Vec::is_empty")]
    message: Vec<Text>,
}

impl TryFrom<MessageContentSerDeHelper> for MessageContent {
    type Error = MessageContentSerdeError;

    fn try_from(helper: MessageContentSerDeHelper) -> Result<Self, Self::Error> {
        if !helper.message.is_empty() {
            Ok(Self(helper.message))
        } else if let Some(text) = helper.text {
            Ok(Self::plain(text))
        } else {
            Err(MessageContentSerdeError::MissingField("m.message".into()))
        }
    }
}

impl Serialize for MessageContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut st = serializer.serialize_struct("MessageContent", 1)?;
        let variants = self.variants();
        if variants.len() == 1 && variants[0].mimetype == "text/plain" {
            st.serialize_field("org.matrix.msc1767.text", &variants[0].body)?;
        } else {
            st.serialize_field("org.matrix.msc1767.message", variants)?;
        }
        st.end()
    }
}
