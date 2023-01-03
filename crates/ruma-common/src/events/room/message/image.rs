use serde::{Deserialize, Serialize};

use crate::{
    events::room::{EncryptedFile, ImageInfo, MediaSource},
    OwnedMxcUri,
};

/// The payload for an image message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.image")]
pub struct ImageMessageEventContent {
    /// A textual representation of the image.
    ///
    /// Could be the alt text of the image, the filename of the image, or some kind of content
    /// description for accessibility e.g. "image attachment".
    pub body: String,

    /// The source of the image.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the image referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,
}

impl ImageMessageEventContent {
    /// Creates a new non-encrypted `ImageMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<ImageInfo>>) -> Self {
        Self { body, source: MediaSource::Plain(url), info }
    }

    /// Creates a new encrypted `ImageMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self { body, source: MediaSource::Encrypted(Box::new(file)), info: None }
    }
}
