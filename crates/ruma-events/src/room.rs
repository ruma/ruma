//! Modules for events in the `m.room` namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use std::{
    collections::{BTreeMap, btree_map},
    fmt,
    ops::Deref,
};

use as_variant::as_variant;
use js_int::UInt;
use ruma_common::{
    OwnedMxcUri,
    serde::{
        Base64, JsonObject,
        base64::{Standard, UrlSafe},
    },
};
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize, de};
use zeroize::Zeroize;

use crate::PrivOwnedStr;

pub mod aliases;
pub mod avatar;
pub mod canonical_alias;
pub mod create;
pub mod encrypted;
mod encrypted_file_serde;
pub mod encryption;
pub mod guest_access;
pub mod history_visibility;
pub mod join_rules;
#[cfg(feature = "unstable-msc4334")]
pub mod language;
pub mod member;
pub mod message;
pub mod name;
pub mod pinned_events;
pub mod policy;
pub mod power_levels;
pub mod redaction;
pub mod server_acl;
pub mod third_party_invite;
mod thumbnail_source_serde;
pub mod tombstone;
pub mod topic;

/// The source of a media file.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_enums)]
pub enum MediaSource {
    /// The MXC URI to the unencrypted media file.
    #[serde(rename = "url")]
    Plain(OwnedMxcUri),

    /// The encryption info of the encrypted media file.
    #[serde(rename = "file")]
    Encrypted(Box<EncryptedFile>),
}

// Custom implementation of `Deserialize`, because serde doesn't guarantee what variant will be
// deserialized for "externally tagged"¹ enums where multiple "tag" fields exist.
//
// ¹ https://serde.rs/enum-representations.html
impl<'de> Deserialize<'de> for MediaSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MediaSourceJsonRepr {
            url: Option<OwnedMxcUri>,
            file: Option<Box<EncryptedFile>>,
        }

        match MediaSourceJsonRepr::deserialize(deserializer)? {
            MediaSourceJsonRepr { url: None, file: None } => Err(de::Error::missing_field("url")),
            // Prefer file if it is set
            MediaSourceJsonRepr { file: Some(file), .. } => Ok(MediaSource::Encrypted(file)),
            MediaSourceJsonRepr { url: Some(url), .. } => Ok(MediaSource::Plain(url)),
        }
    }
}

/// Metadata about an image.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

    /// Metadata about the image referred to in `thumbnail_source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The source of the thumbnail of the image.
    #[serde(flatten, with = "thumbnail_source_serde", skip_serializing_if = "Option::is_none")]
    pub thumbnail_source: Option<MediaSource>,

    /// The [BlurHash](https://blurha.sh) for this image.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,

    /// The [ThumbHash](https://evanw.github.io/thumbhash/) for this image.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.thumbhash", skip_serializing_if = "Option::is_none")]
    pub thumbhash: Option<Base64>,

    /// If this flag is `true`, the original image SHOULD be assumed to be animated. If this flag
    /// is `false`, the original image SHOULD be assumed to NOT be animated.
    ///
    /// If a sending client is unable to determine whether an image is animated, it SHOULD leave
    /// the flag unset.
    ///
    /// Receiving clients MAY use this flag to optimize whether to download the original image
    /// rather than a thumbnail if it is animated, but they SHOULD NOT trust this flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_animated: Option<bool>,
}

impl ImageInfo {
    /// Creates an empty `ImageInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Metadata about a thumbnail.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct EncryptedFile {
    /// The URL to the file.
    pub url: OwnedMxcUri,

    /// Information about the encryption of the file.
    #[serde(flatten)]
    pub info: EncryptedFileInfo,

    /// A map from an algorithm name to a hash of the ciphertext.
    ///
    /// Clients should support the SHA-256 hash.
    pub hashes: EncryptedFileHashes,
}

impl EncryptedFile {
    /// Construct a new `EncryptedFile` with the given URL, encryption info and hashes.
    pub fn new(url: OwnedMxcUri, info: EncryptedFileInfo, hashes: EncryptedFileHashes) -> Self {
        Self { url, info, hashes }
    }
}

/// Information about the encryption of a file.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "v", rename_all = "lowercase")]
pub enum EncryptedFileInfo {
    /// Information about a file encrypted using version 2 of the attachment encryption protocol.
    V2(V2EncryptedFileInfo),

    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomEncryptedFileInfo),
}

impl EncryptedFileInfo {
    /// Get the version of the attachment encryption protocol.
    ///
    /// This matches the `v` field in the serialized data.
    pub fn version(&self) -> &str {
        match self {
            Self::V2(_) => "v2",
            Self::_Custom(info) => &info.v,
        }
    }

    /// Get the data of the attachment encryption protocol, if it doesn't match one of the known
    /// variants.
    pub fn custom_data(&self) -> Option<&JsonObject> {
        as_variant!(self, Self::_Custom(info) => &info.data)
    }
}

impl From<V2EncryptedFileInfo> for EncryptedFileInfo {
    fn from(value: V2EncryptedFileInfo) -> Self {
        Self::V2(value)
    }
}

/// A file encrypted with the AES-CTR algorithm with a 256-bit key.
#[derive(Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct V2EncryptedFileInfo {
    /// The 256-bit key used to encrypt or decrypt the file.
    pub k: Base64<UrlSafe, [u8; 32]>,

    /// The 128-bit unique counter block used by AES-CTR.
    pub iv: Base64<Standard, [u8; 16]>,
}

impl V2EncryptedFileInfo {
    /// Construct a new `V2EncryptedFileInfo` with the given encoded key and initialization vector.
    pub fn new(k: Base64<UrlSafe, [u8; 32]>, iv: Base64<Standard, [u8; 16]>) -> Self {
        Self { k, iv }
    }

    /// Construct a new `V2EncryptedFileInfo` by base64-encoding the given key and initialization
    /// vector bytes.
    pub fn encode(k: [u8; 32], iv: [u8; 16]) -> Self {
        Self::new(Base64::new(k), Base64::new(iv))
    }
}

impl fmt::Debug for V2EncryptedFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("V2EncryptedFileInfo").finish_non_exhaustive()
    }
}

impl Drop for V2EncryptedFileInfo {
    fn drop(&mut self) {
        self.k.zeroize();
    }
}

/// Information about a file encrypted using a custom version of the attachment encryption protocol.
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEncryptedFileInfo {
    /// The version of the protocol.
    v: String,

    /// Extra data about the encryption.
    #[serde(flatten)]
    data: JsonObject,
}

/// A map of [`EncryptedFileHashAlgorithm`] to the associated [`EncryptedFileHash`].
///
/// This type is used to ensure that a supported [`EncryptedFileHash`] always matches the
/// appropriate [`EncryptedFileHashAlgorithm`].
#[derive(Clone, Debug, Default)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct EncryptedFileHashes(BTreeMap<EncryptedFileHashAlgorithm, EncryptedFileHash>);

impl EncryptedFileHashes {
    /// Construct an empty `EncryptedFileHashes`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an `EncryptedFileHashes` that includes the given SHA-256 hash.
    pub fn with_sha256(hash: [u8; 32]) -> Self {
        std::iter::once(EncryptedFileHash::Sha256(Base64::new(hash))).collect()
    }

    /// Insert the given [`EncryptedFileHash`].
    ///
    /// If a map with the same [`EncryptedFileHashAlgorithm`] was already present, it is returned.
    pub fn insert(&mut self, hash: EncryptedFileHash) -> Option<EncryptedFileHash> {
        self.0.insert(hash.algorithm(), hash)
    }
}

impl Deref for EncryptedFileHashes {
    type Target = BTreeMap<EncryptedFileHashAlgorithm, EncryptedFileHash>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<EncryptedFileHash> for EncryptedFileHashes {
    fn from_iter<T: IntoIterator<Item = EncryptedFileHash>>(iter: T) -> Self {
        Self(iter.into_iter().map(|hash| (hash.algorithm(), hash)).collect())
    }
}

impl Extend<EncryptedFileHash> for EncryptedFileHashes {
    fn extend<T: IntoIterator<Item = EncryptedFileHash>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(|hash| (hash.algorithm(), hash)));
    }
}

impl IntoIterator for EncryptedFileHashes {
    type Item = EncryptedFileHash;
    type IntoIter = btree_map::IntoValues<EncryptedFileHashAlgorithm, EncryptedFileHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_values()
    }
}

/// An algorithm used to generate the hash of an [`EncryptedFile`].
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum EncryptedFileHashAlgorithm {
    /// The SHA-256 algorithm
    Sha256,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The hash of an encrypted file's ciphertext.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum EncryptedFileHash {
    /// A hash computed with the SHA-256 algorithm.
    Sha256(Base64<Standard, [u8; 32]>),

    #[doc(hidden)]
    _Custom(CustomEncryptedFileHash),
}

impl EncryptedFileHash {
    /// The key that was used to group this map.
    pub fn algorithm(&self) -> EncryptedFileHashAlgorithm {
        match self {
            Self::Sha256(_) => EncryptedFileHashAlgorithm::Sha256,
            Self::_Custom(custom) => custom.algorithm.as_str().into(),
        }
    }

    /// Get a reference to the decoded bytes of the hash.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Sha256(hash) => hash.as_bytes(),
            Self::_Custom(custom) => custom.hash.as_bytes(),
        }
    }

    /// Get the decoded bytes of the hash.
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Sha256(hash) => hash.into_inner().into(),
            Self::_Custom(custom) => custom.hash.into_inner(),
        }
    }
}

/// A map of results grouped by custom key type.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct CustomEncryptedFileHash {
    /// The algorithm that was used to generate the hash.
    algorithm: String,

    /// The hash.
    hash: Base64,
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::owned_mxc_uri;
    use serde::Deserialize;
    use serde_json::{from_value as from_json_value, json};

    use super::{EncryptedFile, MediaSource, V2EncryptedFileInfo};
    use crate::room::EncryptedFileHashes;

    #[derive(Deserialize)]
    struct MsgWithAttachment {
        #[allow(dead_code)]
        body: String,
        #[serde(flatten)]
        source: MediaSource,
    }

    #[test]
    fn prefer_encrypted_attachment_over_plain() {
        let msg: MsgWithAttachment = from_json_value(json!({
            "body": "",
            "file": EncryptedFile::new(
                owned_mxc_uri!("mxc://localhost/encryptedfile"),
                V2EncryptedFileInfo::encode([0;32], [1;16]).into(),
                EncryptedFileHashes::new(),
            ),
            "url": "mxc://localhost/file",
        }))
        .unwrap();

        assert_matches!(msg.source, MediaSource::Encrypted(_));
    }
}
