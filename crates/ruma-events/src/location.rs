//! Types for extensible location message events ([MSC3488]).
//!
//! [MSC3488]: https://github.com/matrix-org/matrix-spec-proposals/pull/3488

use js_int::UInt;
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

mod zoomlevel_serde;

use ruma_common::MilliSecondsSinceUnixEpoch;

use super::{message::TextContentBlock, room::message::Relation};
use crate::PrivOwnedStr;

/// The payload for an extensible location message.
///
/// This is the new primary type introduced in [MSC3488] and should only be sent in rooms with a
/// version that supports it. See the documentation of the [`message`] module for more information.
///
/// [MSC3488]: https://github.com/matrix-org/matrix-spec-proposals/pull/3488
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.location", kind = MessageLike, without_relation)]
pub struct LocationEventContent {
    /// The text representation of the message.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

    /// The location info of the message.
    #[serde(rename = "m.location")]
    pub location: LocationContent,

    /// The asset this message refers to.
    #[serde(default, rename = "m.asset", skip_serializing_if = "ruma_common::serde::is_default")]
    pub asset: AssetContent,

    /// The timestamp this message refers to.
    #[serde(rename = "m.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,

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
    pub relates_to: Option<Relation<LocationEventContentWithoutRelation>>,
}

impl LocationEventContent {
    /// Creates a new `LocationEventContent` with the given fallback representation and location.
    pub fn new(text: TextContentBlock, location: LocationContent) -> Self {
        Self {
            text,
            location,
            asset: Default::default(),
            ts: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// Creates a new `LocationEventContent` with the given plain text fallback representation and
    /// location.
    pub fn with_plain_text(plain_text: impl Into<String>, location: LocationContent) -> Self {
        Self {
            text: TextContentBlock::plain(plain_text),
            location,
            asset: Default::default(),
            ts: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }
}

/// Location content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LocationContent {
    /// A `geo:` URI representing the location.
    ///
    /// See [RFC 5870](https://datatracker.ietf.org/doc/html/rfc5870) for more details.
    pub uri: String,

    /// The description of the location.
    ///
    /// It should be used to label the location on a map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A zoom level to specify the displayed area size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom_level: Option<ZoomLevel>,
}

impl LocationContent {
    /// Creates a new `LocationContent` with the given geo URI.
    pub fn new(uri: String) -> Self {
        Self { uri, description: None, zoom_level: None }
    }
}

/// An error encountered when trying to convert to a `ZoomLevel`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ZoomLevelError {
    /// The value is higher than [`ZoomLevel::MAX`].
    #[error("value too high")]
    TooHigh,
}

/// A zoom level.
///
/// This is an integer between 0 and 20 as defined in the [OpenStreetMap Wiki].
///
/// [OpenStreetMap Wiki]: https://wiki.openstreetmap.org/wiki/Zoom_levels
#[derive(Clone, Debug, Serialize)]
pub struct ZoomLevel(UInt);

impl ZoomLevel {
    /// The smallest value of a `ZoomLevel`, 0.
    pub const MIN: u8 = 0;

    /// The largest value of a `ZoomLevel`, 20.
    pub const MAX: u8 = 20;

    /// Creates a new `ZoomLevel` with the given value.
    pub fn new(value: u8) -> Option<Self> {
        if value > Self::MAX {
            None
        } else {
            Some(Self(value.into()))
        }
    }

    /// The value of this `ZoomLevel`.
    pub fn get(&self) -> UInt {
        self.0
    }
}

impl TryFrom<u8> for ZoomLevel {
    type Error = ZoomLevelError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(ZoomLevelError::TooHigh)
    }
}

/// Asset content.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AssetContent {
    /// The type of asset being referred to.
    #[serde(rename = "type")]
    pub type_: AssetType,
}

impl AssetContent {
    /// Creates a new default `AssetContent`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The type of an asset.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = "m.snake_case")]
#[non_exhaustive]
pub enum AssetType {
    /// The asset is the sender of the event.
    #[default]
    #[ruma_enum(rename = "m.self")]
    Self_,

    /// The asset is a location pinned by the sender.
    Pin,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
