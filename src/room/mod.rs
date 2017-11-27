//! Modules for events in the *m.room* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

pub mod aliases;
pub mod avatar;
pub mod canonical_alias;
pub mod create;
pub mod guest_access;
pub mod history_visibility;
pub mod join_rules;
pub mod member;
pub mod message;
pub mod name;
pub mod pinned_events;
pub mod power_levels;
pub mod redaction;
pub mod third_party_invite;
pub mod topic;

/// Metadata about an image.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ImageInfo {
    /// The height of the image in pixels.
    #[serde(rename="h")]
    pub height: u64,
    /// The MIME type of the image, e.g. "image/png."
    pub mimetype: String,
    /// The file size of the image in bytes.
    pub size: u64,
    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if="Option::is_none")]
    pub thumbnail_info: Option<ThumbnailInfo>,
    /// The URL to the thumbnail of the image.
    #[serde(skip_serializing_if="Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// The width of the image in pixels.
    #[serde(rename="w")]
    pub width: u64,
}

/// Metadata about a thumbnail.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ThumbnailInfo {
    /// The height of the thumbnail in pixels.
    #[serde(rename="h")]
    pub height: u64,
    /// The MIME type of the thumbnail, e.g. "image/png."
    pub mimetype: String,
    /// The file size of the thumbnail in bytes.
    pub size: u64,
    /// The width of the thumbnail in pixels.
    #[serde(rename="w")]
    pub width: u64,
}
