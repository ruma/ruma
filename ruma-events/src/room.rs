//! Modules for events in the *m.room* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use std::collections::BTreeMap;

use js_int::UInt;
use serde::{Deserialize, Serialize};

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
pub mod relationships;
pub mod server_acl;
pub mod third_party_invite;
pub mod tombstone;
pub mod topic;

/// Metadata about an image.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

    /// The URL to the thumbnail of the image. Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on the encrypted thumbnail image. Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,

    /// The [BlurHash](https://blurha.sh) for this image.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "xyz.amorgan.blurhash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

/// Metadata about a thumbnail.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// A file sent to a room with end-to-end encryption enabled.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedFile {
    /// The URL to the file.
    pub url: String,

    /// A [JSON Web Key](https://tools.ietf.org/html/rfc7517#appendix-A.3) object.
    pub key: JsonWebKey,

    /// The 128-bit unique counter block used by AES-CTR, encoded as unpadded base64.
    pub iv: String,

    /// A map from an algorithm name to a hash of the ciphertext, encoded as unpadded base64.
    /// Clients should support the SHA-256 hash, which uses the key sha256.
    pub hashes: BTreeMap<String, String>,

    /// Version of the encrypted attachments protocol. Must be `v2`.
    pub v: String,
}

/// A [JSON Web Key](https://tools.ietf.org/html/rfc7517#appendix-A.3) object.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonWebKey {
    /// Key type. Must be `oct`.
    pub kty: String,

    /// Key operations. Must at least contain `encrypt` and `decrypt`.
    pub key_ops: Vec<String>,

    /// Required. Algorithm. Must be `A256CTR`.
    pub alg: String,

    /// The key, encoded as urlsafe unpadded base64.
    pub k: String,

    /// Extractable. Must be `true`. This is a
    /// [W3C extension](https://w3c.github.io/webcrypto/#iana-section-jwk).
    pub ext: bool,
}
