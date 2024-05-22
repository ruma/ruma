//! Types for the [`m.sticker`] event.
//!
//! [`m.sticker`]: https://spec.matrix.org/latest/client-server-api/#msticker

use std::fmt;

use ruma_common::OwnedMxcUri;
use ruma_macros::EventContent;
use serde::{
    de::{self, Deserializer, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};

use crate::room::{EncryptedFile, ImageInfo, MediaSource};

/// The content of an `m.sticker` event.
///
/// A sticker message.
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.sticker", kind = MessageLike)]
pub struct StickerEventContent {
    /// A textual representation or associated description of the sticker image.
    ///
    /// This could be the alt text of the original image, or a message to accompany and further
    /// describe the sticker.
    pub body: String,

    /// Metadata about the image referred to in `url` including a thumbnail representation.
    pub info: ImageInfo,

    /// The URL to the sticker image.
    pub url: OwnedMxcUri,

    /// The media source
    #[cfg(not(feature = "compat-encrypted-stickers"))]
    #[serde(skip)]
    pub source: MediaSource,
}

impl StickerEventContent {
    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    #[cfg(not(feature = "compat-encrypted-stickers"))]
    pub fn new(body: String, info: ImageInfo, url: OwnedMxcUri) -> Self {
        Self { body, info, url: url.clone(), source: MediaSource::Plain(url.clone()) }
    }
    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    #[cfg(feature = "compat-encrypted-stickers")]
    pub fn new(body: String, info: ImageInfo, url: OwnedMxcUri) -> Self {
        Self { body, info, url }
    }

    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    #[cfg(not(feature = "compat-encrypted-stickers"))]
    pub fn from_source(
        body: String,
        info: ImageInfo,
        url: OwnedMxcUri,
        source: MediaSource,
    ) -> Self {
        Self { body, info, url, source }
    }
    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    #[cfg(feature = "compat-encrypted-stickers")]
    pub fn from_source(
        body: String,
        info: ImageInfo,
        url: OwnedMxcUri,
        _source: MediaSource,
    ) -> Self {
        Self { body, info, url }
    }
}

impl<'de> Deserialize<'de> for StickerEventContent {
    fn deserialize<D>(deserializer: D) -> Result<StickerEventContent, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default)]
        enum Field {
            Body,
            Info,
            Url,
            File,
            #[default]
            None,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;
                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("'body/info/url/file'")
                    }
                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "body" => Ok(Field::Body),
                            "info" => Ok(Field::Info),
                            "url" => Ok(Field::Url),
                            "file" => Ok(Field::File),
                            _ => Ok(Field::default()),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct StickerEventContentVisitor;

        impl<'de> Visitor<'de> for StickerEventContentVisitor {
            type Value = StickerEventContent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct StickerEventContent")
            }

            fn visit_map<V>(self, mut map: V) -> Result<StickerEventContent, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut body: Option<Result<String, <V as MapAccess<'de>>::Error>> = None;
                let mut info: Option<Result<ImageInfo, <V as MapAccess<'de>>::Error>> = None;
                let mut url: Option<Result<OwnedMxcUri, <V as MapAccess<'de>>::Error>> = None;
                let mut file: Option<Result<EncryptedFile, <V as MapAccess<'de>>::Error>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Body => body = Some(map.next_value()),
                        Field::Info => info = Some(map.next_value()),
                        Field::Url => url = Some(map.next_value()),
                        Field::File => file = Some(map.next_value::<EncryptedFile>()),
                        Field::None => {
                            let _ = Some(map.next_value::<IgnoredAny>());
                        }
                    }
                }
                let body: Result<String, <V as MapAccess<'de>>::Error> =
                    body.ok_or_else(|| de::Error::missing_field("body"))?;
                let info: Result<ImageInfo, <V as MapAccess<'de>>::Error> =
                    info.ok_or_else(|| de::Error::missing_field("info"))?;

                match file {
                    Some(file) => {
                        let file = file.unwrap();
                        Ok(StickerEventContent::from_source(
                            body.unwrap(),
                            info.unwrap(),
                            file.url.clone(),
                            MediaSource::Encrypted(Box::from(file)),
                        ))
                    }
                    None => {
                        Ok(StickerEventContent::new(body.unwrap(), info.unwrap(), url.unwrap()?))
                    }
                }
            }
        }
        const FIELDS: &[&str] = &["body", "info", "url", "file"];
        deserializer.deserialize_struct("StickerEventContent", FIELDS, StickerEventContentVisitor)
    }
}
