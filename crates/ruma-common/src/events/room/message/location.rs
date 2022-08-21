use serde::{Deserialize, Serialize};

use crate::events::room::{MediaSource, ThumbnailInfo};
#[cfg(feature = "unstable-msc3488")]
use crate::{
    events::{
        location::{AssetContent, LocationContent},
        message::MessageContent,
    },
    MilliSecondsSinceUnixEpoch,
};

/// The payload for a location message.
///
/// With the `unstable-msc3488` feature, this type contains the transitional format of
/// [`LocationEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`LocationEventContent`]: crate::events::location::LocationEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.location")]
#[cfg_attr(
    feature = "unstable-msc3488",
    serde(from = "super::content_serde::LocationMessageEventContentDeHelper")
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
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event location info of the message.
    ///
    /// If present, this should be preferred over the `geo_uri` field.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(rename = "org.matrix.msc3488.location", skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationContent>,

    /// Extensible-event asset this message refers to.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(rename = "org.matrix.msc3488.asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<AssetContent>,

    /// Extensible-event timestamp this message refers to.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl LocationMessageEventContent {
    /// Creates a new `LocationMessageEventContent` with the given body and geo URI.
    pub fn new(body: String, geo_uri: String) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3488")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3488")]
            location: Some(LocationContent::new(geo_uri.clone())),
            #[cfg(feature = "unstable-msc3488")]
            asset: None,
            #[cfg(feature = "unstable-msc3488")]
            ts: None,
            body,
            geo_uri,
            info: None,
        }
    }

    /// Create a new `LocationMessageEventContent` with the given message, location info, asset and
    /// timestamp.
    #[cfg(feature = "unstable-msc3488")]
    pub(crate) fn from_extensible_content(
        message: MessageContent,
        location: LocationContent,
        asset: AssetContent,
        ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let geo_uri = location.uri.clone();

        Self {
            message: Some(message),
            location: Some(location),
            asset: Some(asset),
            ts,
            body,
            geo_uri,
            info: None,
        }
    }
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LocationInfo {
    /// The source of a thumbnail of the location.
    #[serde(
        flatten,
        with = "crate::events::room::thumbnail_source_serde",
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
