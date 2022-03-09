//! Modules for events in the `m.room` namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_serde::{base64::UrlSafe, Base64};
use serde::{Deserialize, Serialize};

use crate::MxcUri;

pub mod aliases;
pub mod avatar;
pub mod canonical_alias;
pub mod create;
pub mod encrypted;
pub mod encryption;
pub mod guest_access;
pub mod history_visibility;
pub mod join_rules;
pub mod member;
pub mod message;
pub mod name;
pub mod pinned_events;
pub mod power_levels;
pub mod redaction;
pub mod server_acl;
pub mod third_party_invite;
pub mod tombstone;
pub mod topic;

/// Metadata about an image.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ImageInfo {
    /// The height of the image in pixels.
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the image in pixels.
    #[serde(rename = "w", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The MIME type of the image, e.g. "image/png."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The file size of the image in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL to the thumbnail of the image.
    ///
    /// Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<Box<MxcUri>>,

    /// Information on the encrypted thumbnail image.
    ///
    /// Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,

    /// The [BlurHash](https://blurha.sh) for this image.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

impl ImageInfo {
    /// Creates an empty `ImageInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Metadata about a thumbnail.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailInfo {
    /// The height of the thumbnail in pixels.
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the thumbnail in pixels.
    #[serde(rename = "w", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The MIME type of the thumbnail, e.g. "image/png."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The file size of the thumbnail in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,
}

impl ThumbnailInfo {
    /// Creates an empty `ThumbnailInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A file sent to a room with end-to-end encryption enabled.
///
/// To create an instance of this type, first create a `EncryptedFileInit` and convert it via
/// `EncryptedFile::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct EncryptedFile {
    /// The URL to the file.
    pub url: Box<MxcUri>,

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

/// Initial set of fields of `EncryptedFile`.
///
/// This struct will not be updated even if additional fields are added to `EncryptedFile` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct EncryptedFileInit {
    /// The URL to the file.
    pub url: Box<MxcUri>,

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

impl From<EncryptedFileInit> for EncryptedFile {
    fn from(init: EncryptedFileInit) -> Self {
        let EncryptedFileInit { url, key, iv, hashes, v } = init;
        Self { url, key, iv, hashes, v }
    }
}

/// A [JSON Web Key](https://tools.ietf.org/html/rfc7517#appendix-A.3) object.
///
/// To create an instance of this type, first create a `JsonWebKeyInit` and convert it via
/// `JsonWebKey::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct JsonWebKey {
    /// Key type.
    ///
    /// Must be `oct`.
    pub kty: String,

    /// Key operations.
    ///
    /// Must at least contain `encrypt` and `decrypt`.
    pub key_ops: Vec<String>,

    /// Algorithm.
    ///
    /// Must be `A256CTR`.
    pub alg: String,

    /// The key, encoded as url-safe unpadded base64.
    pub k: Base64<UrlSafe>,

    /// Extractable.
    ///
    /// Must be `true`. This is a
    /// [W3C extension](https://w3c.github.io/webcrypto/#iana-section-jwk).
    pub ext: bool,
}

/// Initial set of fields of `JsonWebKey`.
///
/// This struct will not be updated even if additional fields are added to `JsonWebKey` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct JsonWebKeyInit {
    /// Key type.
    ///
    /// Must be `oct`.
    pub kty: String,

    /// Key operations.
    ///
    /// Must at least contain `encrypt` and `decrypt`.
    pub key_ops: Vec<String>,

    /// Algorithm.
    ///
    /// Must be `A256CTR`.
    pub alg: String,

    /// The key, encoded as url-safe unpadded base64.
    pub k: Base64<UrlSafe>,

    /// Extractable.
    ///
    /// Must be `true`. This is a
    /// [W3C extension](https://w3c.github.io/webcrypto/#iana-section-jwk).
    pub ext: bool,
}

impl From<JsonWebKeyInit> for JsonWebKey {
    fn from(init: JsonWebKeyInit) -> Self {
        let JsonWebKeyInit { kty, key_ops, alg, k, ext } = init;
        Self { kty, key_ops, alg, k, ext }
    }
}
