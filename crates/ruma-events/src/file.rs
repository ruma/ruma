//! Types for extensible file message events ([MSC3551]).
//!
//! [MSC3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{serde::Base64, OwnedMxcUri};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    message::TextContentBlock,
    room::{message::Relation, EncryptedFile, JsonWebKey},
};

/// The payload for an extensible file message.
///
/// This is the new primary type introduced in [MSC3551] and should only be sent in rooms with a
/// version that supports it. See the documentation of the [`message`] module for more information.
///
/// [MSC3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc1767.file", kind = MessageLike, without_relation)]
pub struct FileEventContent {
    /// The text representation of the message.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

    /// The file content of the message.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file: FileContentBlock,

    /// The caption of the message, if any.
    #[serde(rename = "org.matrix.msc1767.caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<CaptionContentBlock>,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,

    /// Information about related messages.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<FileEventContentWithoutRelation>>,
}

impl FileEventContent {
    /// Creates a new non-encrypted `FileEventContent` with the given fallback representation, url
    /// and file info.
    pub fn plain(text: TextContentBlock, url: OwnedMxcUri, name: String) -> Self {
        Self {
            text,
            file: FileContentBlock::plain(url, name),
            caption: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// Creates a new non-encrypted `FileEventContent` with the given plain text fallback
    /// representation, url and name.
    pub fn plain_with_plain_text(
        plain_text: impl Into<String>,
        url: OwnedMxcUri,
        name: String,
    ) -> Self {
        Self {
            text: TextContentBlock::plain(plain_text),
            file: FileContentBlock::plain(url, name),
            caption: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// Creates a new encrypted `FileEventContent` with the given fallback representation, url,
    /// name and encryption info.
    pub fn encrypted(
        text: TextContentBlock,
        url: OwnedMxcUri,
        name: String,
        encryption_info: EncryptedContent,
    ) -> Self {
        Self {
            text,
            file: FileContentBlock::encrypted(url, name, encryption_info),
            caption: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// Creates a new encrypted `FileEventContent` with the given plain text fallback
    /// representation, url, name and encryption info.
    pub fn encrypted_with_plain_text(
        plain_text: impl Into<String>,
        url: OwnedMxcUri,
        name: String,
        encryption_info: EncryptedContent,
    ) -> Self {
        Self {
            text: TextContentBlock::plain(plain_text),
            file: FileContentBlock::encrypted(url, name, encryption_info),
            caption: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }
}

/// A block for file content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FileContentBlock {
    /// The URL to the file.
    pub url: OwnedMxcUri,

    /// The original filename of the uploaded file.
    pub name: String,

    /// The mimetype of the file, e.g. "application/msword".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the file in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Information on the encrypted file.
    ///
    /// Required if the file is encrypted.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub encryption_info: Option<Box<EncryptedContent>>,
}

impl FileContentBlock {
    /// Creates a new non-encrypted `FileContentBlock` with the given url and name.
    pub fn plain(url: OwnedMxcUri, name: String) -> Self {
        Self { url, name, mimetype: None, size: None, encryption_info: None }
    }

    /// Creates a new encrypted `FileContentBlock` with the given url, name and encryption info.
    pub fn encrypted(url: OwnedMxcUri, name: String, encryption_info: EncryptedContent) -> Self {
        Self {
            url,
            name,
            mimetype: None,
            size: None,
            encryption_info: Some(Box::new(encryption_info)),
        }
    }

    /// Whether the file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_info.is_some()
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
        let EncryptedFile { key, iv, hashes, v, .. } = encrypted;
        Self { key: key.to_owned(), iv: iv.to_owned(), hashes: hashes.to_owned(), v: v.to_owned() }
    }
}

/// A block for caption content.
///
/// A caption is usually a text message that should be displayed next to some media content.
///
/// To construct a `CaptionContentBlock` with a custom [`TextContentBlock`], convert it with
/// `CaptionContentBlock::from()` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CaptionContentBlock {
    /// The text message of the caption.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,
}

impl CaptionContentBlock {
    /// A convenience constructor to create a plain text caption content block.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { text: TextContentBlock::plain(body) }
    }

    /// A convenience constructor to create an HTML caption content block.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { text: TextContentBlock::html(body, html_body) }
    }

    /// A convenience constructor to create a caption content block from Markdown.
    ///
    /// The content includes an HTML message if some Markdown formatting was detected, otherwise
    /// only a plain text message is included.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { text: TextContentBlock::markdown(body) }
    }
}

impl From<TextContentBlock> for CaptionContentBlock {
    fn from(text: TextContentBlock) -> Self {
        Self { text }
    }
}
