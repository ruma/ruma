//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use serde::{ser::SerializeStruct, Deserialize, Serialize};

use super::{MessageContent, Text, TryFromExtensibleError};

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
    type Error = TryFromExtensibleError;

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
            Err(TryFromExtensibleError::MissingField("m.message or m.text".to_owned()))
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

pub(crate) mod as_vec {
    use serde::{de, ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    use crate::events::message::{MessageContent, Text};

    /// Serializes a `Option<MessageContent>` as a `Vec<Text>`.
    pub fn serialize<S>(content: &Option<MessageContent>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(content) = content {
            let mut seq = serializer.serialize_seq(Some(content.len()))?;
            for e in content.iter() {
                seq.serialize_element(e)?;
            }
            seq.end()
        } else {
            serializer.serialize_seq(Some(0))?.end()
        }
    }

    /// Deserializes a `Vec<Text>` to an `Option<MessageContent>`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<MessageContent>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Vec<Text>>::deserialize(deserializer).and_then(|content| {
            content.map(MessageContent::new).ok_or_else(|| {
                de::Error::invalid_value(de::Unexpected::Other("empty array"), &"a non-empty array")
            })
        })
    }
}
