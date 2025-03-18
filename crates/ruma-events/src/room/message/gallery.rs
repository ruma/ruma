use ruma_common::serde::JsonObject;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{
    AudioMessageEventContent, CustomEventContent, FileMessageEventContent, FormattedBody,
    ImageMessageEventContent, VideoMessageEventContent,
};

/// The payload for a gallery message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "msgtype", rename = "dm.filament.gallery")]
pub struct GalleryMessageEventContent {
    /// A human-readable description of the gallery.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// Item types for the media in the gallery.
    pub itemtypes: Vec<GalleryItemType>,
}

impl GalleryMessageEventContent {
    /// Creates a new `GalleryMessageEventContent`.
    pub fn new(
        body: String,
        formatted: Option<FormattedBody>,
        itemtypes: Vec<GalleryItemType>,
    ) -> Self {
        Self { body, formatted, itemtypes }
    }
}

/// The content that is specific to each gallery item type variant.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum GalleryItemType {
    /// An audio item.
    Audio(AudioMessageEventContent),

    /// A file item.
    File(FileMessageEventContent),

    /// An image item.
    Image(ImageMessageEventContent),

    /// A video item.
    Video(VideoMessageEventContent),

    /// A custom item.
    #[doc(hidden)]
    _Custom(CustomEventContent),
}

impl GalleryItemType {
    /// Creates a new `GalleryItemType`.
    ///
    /// The `itemtype` and `body` are required fields.
    /// Additionally it's possible to add arbitrary key/value pairs to the event content for custom
    /// item types through the `data` map.
    ///
    /// Prefer to use the public variants of `GalleryItemType` where possible; this constructor is
    /// meant be used for unsupported item types only and does not allow setting arbitrary data
    /// for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `itemtype` is known and serialization of `data` to the corresponding
    /// `GalleryItemType` variant fails.
    pub fn new(itemtype: &str, body: String, data: JsonObject) -> serde_json::Result<Self> {
        fn deserialize_variant<T: DeserializeOwned>(
            body: String,
            mut obj: JsonObject,
        ) -> serde_json::Result<T> {
            obj.insert("body".into(), body.into());
            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match itemtype {
            "m.audio" => Self::Audio(deserialize_variant(body, data)?),
            "m.file" => Self::File(deserialize_variant(body, data)?),
            "m.image" => Self::Image(deserialize_variant(body, data)?),
            "m.video" => Self::Video(deserialize_variant(body, data)?),
            _ => Self::_Custom(CustomEventContent { msgtype: itemtype.to_owned(), body, data }),
        })
    }

    /// Returns a reference to the `itemtype` string.
    pub fn itemtype(&self) -> &str {
        match self {
            Self::Audio(_) => "m.audio",
            Self::File(_) => "m.file",
            Self::Image(_) => "m.image",
            Self::Video(_) => "m.video",
            Self::_Custom(c) => &c.msgtype,
        }
    }

    /// Return a reference to the itemtype body.
    pub fn body(&self) -> &str {
        match self {
            GalleryItemType::Audio(m) => &m.body,
            GalleryItemType::File(m) => &m.body,
            GalleryItemType::Image(m) => &m.body,
            GalleryItemType::Video(m) => &m.body,
            GalleryItemType::_Custom(m) => &m.body,
        }
    }

    /// Return a reference to the itemtype formatted body.
    pub fn formatted(&self) -> &Option<FormattedBody> {
        match self {
            GalleryItemType::Audio(m) => &m.formatted,
            GalleryItemType::File(m) => &m.formatted,
            GalleryItemType::Image(m) => &m.formatted,
            GalleryItemType::Video(m) => &m.formatted,
            GalleryItemType::_Custom(_) => &None,
        }
    }
}
