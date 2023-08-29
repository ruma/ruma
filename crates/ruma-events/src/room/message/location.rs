#[cfg(feature = "unstable-msc3488")]
use ruma_common::MilliSecondsSinceUnixEpoch;
use serde::{Deserialize, Serialize};

use crate::room::{MediaSource, ThumbnailInfo};
#[cfg(feature = "unstable-msc3488")]
use crate::{
    location::{AssetContent, AssetType, LocationContent},
    message::{TextContentBlock, TextRepresentation},
};

/// The payload for a location message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.location")]
#[cfg_attr(
    feature = "unstable-msc3488",
    serde(
        from = "super::content_serde::msc3488::LocationMessageEventContentSerDeHelper",
        into = "super::content_serde::msc3488::LocationMessageEventContentSerDeHelper"
    )
)]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK", or some kind of content
    /// description for accessibility, e.g. "location attachment".
    pub body: String,

    /// A geo URI representing the location.
    pub geo_uri: String,

    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<LocationInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3488")]
    pub message: Option<TextContentBlock>,

    /// Extensible-event location info of the message.
    ///
    /// If present, this should be preferred over the `geo_uri` field.
    #[cfg(feature = "unstable-msc3488")]
    pub location: Option<LocationContent>,

    /// Extensible-event asset this message refers to.
    #[cfg(feature = "unstable-msc3488")]
    pub asset: Option<AssetContent>,

    /// Extensible-event timestamp this message refers to.
    #[cfg(feature = "unstable-msc3488")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl LocationMessageEventContent {
    /// Creates a new `LocationMessageEventContent` with the given body and geo URI.
    pub fn new(body: String, geo_uri: String) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3488")]
            message: Some(vec![TextRepresentation::plain(&body)].into()),
            #[cfg(feature = "unstable-msc3488")]
            location: Some(LocationContent::new(geo_uri.clone())),
            #[cfg(feature = "unstable-msc3488")]
            asset: Some(AssetContent::default()),
            #[cfg(feature = "unstable-msc3488")]
            ts: None,
            body,
            geo_uri,
            info: None,
        }
    }

    /// Set the asset type of this `LocationMessageEventContent`.
    #[cfg(feature = "unstable-msc3488")]
    pub fn with_asset_type(mut self, asset: AssetType) -> Self {
        self.asset = Some(AssetContent { type_: asset });
        self
    }

    /// Set the timestamp of this `LocationMessageEventContent`.
    #[cfg(feature = "unstable-msc3488")]
    pub fn with_ts(mut self, ts: MilliSecondsSinceUnixEpoch) -> Self {
        self.ts = Some(ts);
        self
    }

    /// Get the `geo:` URI of this `LocationMessageEventContent`.
    pub fn geo_uri(&self) -> &str {
        #[cfg(feature = "unstable-msc3488")]
        if let Some(uri) = self.location.as_ref().map(|l| &l.uri) {
            return uri;
        }

        &self.geo_uri
    }

    /// Get the plain text representation of this `LocationMessageEventContent`.
    pub fn plain_text_representation(&self) -> &str {
        #[cfg(feature = "unstable-msc3488")]
        if let Some(text) = self.message.as_ref().and_then(|m| m.find_plain()) {
            return text;
        }

        &self.body
    }

    /// Get the asset type of this `LocationMessageEventContent`.
    #[cfg(feature = "unstable-msc3488")]
    pub fn asset_type(&self) -> AssetType {
        self.asset.as_ref().map(|a| a.type_.clone()).unwrap_or_default()
    }
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LocationInfo {
    /// The source of a thumbnail of the location.
    #[serde(
        flatten,
        with = "crate::room::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,

    /// Metadata about the image referred to in `thumbnail_source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,
}

impl LocationInfo {
    /// Creates an empty `LocationInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}
