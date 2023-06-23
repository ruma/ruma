//! Serde for old versions of MSC1767 still used in some types ([spec]).
//!
//! [spec]: https://github.com/matrix-org/matrix-spec-proposals/blob/d6046d8402e7a3c7a4fcbc9da16ea9bad5968992/proposals/1767-extensible-events.md

use serde::{Deserialize, Serialize};

use super::{TextContentBlock, TextRepresentation};

#[derive(Default, Serialize, Deserialize)]
pub(in crate::events) struct MessageContentBlockSerDeHelper {
    /// Plain text short form.
    #[serde(rename = "org.matrix.msc1767.text", skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// HTML short form.
    #[serde(rename = "org.matrix.msc1767.html", skip_serializing_if = "Option::is_none")]
    html: Option<String>,

    /// Long form.
    #[serde(rename = "org.matrix.msc1767.message", skip_serializing_if = "Option::is_none")]
    message: Option<Vec<TextRepresentation>>,
}

impl TryFrom<MessageContentBlockSerDeHelper> for TextContentBlock {
    type Error = &'static str;

    fn try_from(value: MessageContentBlockSerDeHelper) -> Result<Self, Self::Error> {
        let MessageContentBlockSerDeHelper { text, html, message } = value;

        if let Some(message) = message {
            Ok(Self(message))
        } else {
            let message: Vec<_> = html
                .map(TextRepresentation::html)
                .into_iter()
                .chain(text.map(TextRepresentation::plain))
                .collect();
            if !message.is_empty() {
                Ok(Self(message))
            } else {
                Err("missing at least one of fields `org.matrix.msc1767.text`, `org.matrix.msc1767.html` or `org.matrix.msc1767.message`")
            }
        }
    }
}

impl From<TextContentBlock> for MessageContentBlockSerDeHelper {
    fn from(value: TextContentBlock) -> Self {
        let has_shortcut =
            |message: &TextRepresentation| matches!(&*message.mimetype, "text/plain" | "text/html");

        if value.iter().all(has_shortcut) {
            let mut helper = Self::default();

            for message in value.0.into_iter() {
                if message.mimetype == "text/plain" {
                    helper.text = Some(message.body);
                } else if message.mimetype == "text/html" {
                    helper.html = Some(message.body);
                }
            }

            helper
        } else {
            Self { message: Some(value.0), ..Default::default() }
        }
    }
}
