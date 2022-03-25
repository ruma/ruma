//! Types for extensible file message events ([MSC3551]).
//!
//! [MSC3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    message::MessageContent,
    room::{message::Relation, JsonWebKey},
};
use crate::{serde::Base64, MxcUri};

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

/// Information about a file content.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FileContentInfo {
    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The mimetype of the file, e.g. “application/msword”.
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
}

/// File content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FileContent {
    /// The URL to the file.
    pub url: Box<MxcUri>,

    /// Information about the uploaded file.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileContentInfo>>,

    /// Information on the encrypted file.
    ///
    /// Required if file is encrypted.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub encryption_info: Option<Box<EncryptedContent>>,
}

impl FileContent {
    /// Creates a new non-encrypted `FileContent` with the given url and file info.
    pub fn plain(url: Box<MxcUri>, info: Option<Box<FileContentInfo>>) -> Self {
        Self { url, info, encryption_info: None }
    }

    /// Creates a new encrypted `FileContent` with the given url, encryption info and file info.
    pub fn encrypted(
        url: Box<MxcUri>,
        encryption_info: EncryptedContent,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self { url, info, encryption_info: Some(Box::new(encryption_info)) }
    }

    /// Whether the file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_info.is_some()
    }
}

/// The payload for an extensible text message.
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

    /// Information about related messages for [rich replies].
    ///
    /// [rich replies]: https://spec.matrix.org/v1.2/client-server-api/#rich-replies
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl FileEventContent {
    /// Creates a new non-encrypted `FileEventContent` with the given plain text message, url and
    /// file info.
    pub fn plain(
        message: impl Into<String>,
        url: Box<MxcUri>,
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
        url: Box<MxcUri>,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self { message, file: FileContent::plain(url, info), relates_to: None }
    }

    /// Creates a new encrypted `FileEventContent` with the given plain text message, url,
    /// encryption info and file info.
    pub fn encrypted(
        message: impl Into<String>,
        url: Box<MxcUri>,
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
        url: Box<MxcUri>,
        encryption_info: EncryptedContent,
        info: Option<Box<FileContentInfo>>,
    ) -> Self {
        Self { message, file: FileContent::encrypted(url, encryption_info, info), relates_to: None }
    }
}
