//! Types for extensible file message events ([MSC3551]).
//!
//! [MSC3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    message::MessageContent,
    room::{
        message::{AudioInfo, FileInfo, FileMessageEventContent, Relation, VideoInfo},
        EncryptedFile, ImageInfo, JsonWebKey, MediaSource,
    },
};
use crate::{serde::Base64, OwnedMxcUri};

/// The payload for an extensible file message.
///
/// This is the new primary type introduced in [MSC3551] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `FileEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::File`]. You can convert it back with
/// [`FileEventContent::from_file_room_message()`].
///
/// [MSC3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::File`]: super::room::message::MessageType::File
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.file", kind = MessageLike)]
pub struct FileEventContent {
    /// The text representation of the message.
    #[serde(flatten)]
    pub message: MessageContent,

    /// The file content of the message.
    #[serde(rename = "m.file")]
    pub file: FileContent,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl FileEventContent {
    /// Creates a new non-encrypted `FileEventContent` with the given plain text message, url and
    /// file info.
    pub fn plain(
        message: impl Into<String>,
        url: OwnedMxcUri,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self {
            message: MessageContent::plain(message),
            file: FileContent::plain(url, info),
            relates_to: None,
        }
    }

    /// Creates a new non-encrypted `FileEventContent` with the given message, url and
    /// file info.
    pub fn plain_message(
        message: MessageContent,
        url: OwnedMxcUri,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self { message, file: FileContent::plain(url, info), relates_to: None }
    }

    /// Creates a new encrypted `FileEventContent` with the given plain text message, url,
    /// encryption info and file info.
    pub fn encrypted(
        message: impl Into<String>,
        url: OwnedMxcUri,
        encryption_info: EncryptedContent,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self {
            message: MessageContent::plain(message),
            file: FileContent::encrypted(url, encryption_info, info),
            relates_to: None,
        }
    }

    /// Creates a new encrypted `FileEventContent` with the given message, url,
    /// encryption info and file info.
    pub fn encrypted_message(
        message: MessageContent,
        url: OwnedMxcUri,
        encryption_info: EncryptedContent,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self { message, file: FileContent::encrypted(url, encryption_info, info), relates_to: None }
    }

    /// Create a new `FileEventContent` from the given `FileMessageEventContent` and optional
    /// relation.
    pub fn from_file_room_message(
        content: FileMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Self {
        let FileMessageEventContent { body, filename, source, info, message, file } = content;

        let message = message.unwrap_or_else(|| MessageContent::plain(body));
        let file = file.unwrap_or_else(|| {
            FileContent::from_room_message_content(source, info.as_deref(), filename)
        });

        Self { message, file, relates_to }
    }
}

/// File content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FileContent {
    /// The URL to the file.
    pub url: OwnedMxcUri,

    /// Information about the uploaded file.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileContentInfo>>,

    /// Information on the encrypted file.
    ///
    /// Required if the file is encrypted.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub encryption_info: Option<Box<EncryptedContent>>,
}

impl FileContent {
    /// Creates a new non-encrypted `FileContent` with the given url and file info.
    pub fn plain(url: OwnedMxcUri, info: Option<Box<FileContentInfo>>) -> Self {
        Self { url, info, encryption_info: None }
    }

    /// Creates a new encrypted `FileContent` with the given url, encryption info and file info.
    pub fn encrypted(
        url: OwnedMxcUri,
        encryption_info: EncryptedContent,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self { url, info, encryption_info: Some(Box::new(encryption_info)) }
    }

    /// Create a new `FileContent` with the given media source, file info and filename.
    pub fn from_room_message_content(
        source: MediaSource,
        info: Option<impl Into<FileContentInfo>>,
        filename: Option<String>,
    ) -> Self {
        let (url, encryption_info) = match source {
            MediaSource::Plain(url) => (url, None),
            MediaSource::Encrypted(file) => (file.url.clone(), Some(Box::new((&*file).into()))),
        };
        let info = FileContentInfo::from_room_message_content(info, filename).map(Box::new);

        Self { url, encryption_info, info }
    }

    /// Whether the file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_info.is_some()
    }
}

/// Information about a file content.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FileContentInfo {
    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The mimetype of the file, e.g. "application/msword".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the file in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,
}

impl FileContentInfo {
    /// Creates an empty `FileContentInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `FileContentInfo` with the given file info and filename.
    ///
    /// Returns `None` if both parameters are `None`.
    pub fn from_room_message_content(
        info: Option<impl Into<FileContentInfo>>,
        filename: Option<String>,
    ) -> Option<Self> {
        if filename.is_none() && info.is_none() {
            None
        } else {
            let mut info: Self = info.map(Into::into).unwrap_or_default();
            info.name = filename;
            Some(info)
        }
    }
}

impl From<&AudioInfo> for FileContentInfo {
    fn from(info: &AudioInfo) -> Self {
        let AudioInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned(), ..Default::default() }
    }
}

impl From<&FileInfo> for FileContentInfo {
    fn from(info: &FileInfo) -> Self {
        let FileInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned(), ..Default::default() }
    }
}

impl From<&ImageInfo> for FileContentInfo {
    fn from(info: &ImageInfo) -> Self {
        let ImageInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned(), ..Default::default() }
    }
}

impl From<&VideoInfo> for FileContentInfo {
    fn from(info: &VideoInfo) -> Self {
        let VideoInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned(), ..Default::default() }
    }
}

/// The encryption info of a file sent to a room with end-to-end encryption enabled.
///
/// To create an instance of this type, first create a `EncryptedContentInit` and convert it via
/// `EncryptedContent::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct EncryptedContent {
    /// A [JSON Web Key](https://tools.ietf.org/html/rfc7517#appendix-A.3) object.
    pub key: JsonWebKey,

    /// The 128-bit unique counter block used by AES-CTR, encoded as unpadded base64.
    pub iv: Base64,

    /// A map from an algorithm name to a hash of the ciphertext, encoded as unpadded base64.
    ///
    /// Clients should support the SHA-256 hash, which uses the key sha256.
    pub hashes: BTreeMap<String, Base64>,

    /// Version of the encrypted attachments protocol.
    ///
    /// Must be `v2`.
    pub v: String,
}

/// Initial set of fields of `EncryptedContent`.
///
/// This struct will not be updated even if additional fields are added to `EncryptedContent` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct EncryptedContentInit {
    /// A [JSON Web Key](https://tools.ietf.org/html/rfc7517#appendix-A.3) object.
    pub key: JsonWebKey,

    /// The 128-bit unique counter block used by AES-CTR, encoded as unpadded base64.
    pub iv: Base64,

    /// A map from an algorithm name to a hash of the ciphertext, encoded as unpadded base64.
    ///
    /// Clients should support the SHA-256 hash, which uses the key sha256.
    pub hashes: BTreeMap<String, Base64>,

    /// Version of the encrypted attachments protocol.
    ///
    /// Must be `v2`.
    pub v: String,
}

impl From<EncryptedContentInit> for EncryptedContent {
    fn from(init: EncryptedContentInit) -> Self {
        let EncryptedContentInit { key, iv, hashes, v } = init;
        Self { key, iv, hashes, v }
    }
}

impl From<&EncryptedFile> for EncryptedContent {
    fn from(encrypted: &EncryptedFile) -> Self {
        let EncryptedFile { key, iv, hashes, v, .. } = encrypted.to_owned();
        Self { key, iv, hashes, v }
    }
}
