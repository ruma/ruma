//! Types for extensible live location message events ([MSC3489]).
//!
//! [MSC3489]: https://github.com/matrix-org/matrix-spec-proposals/pull/3489


use serde::{Deserialize, Serialize};
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_macros::EventContent;
use crate::location::{AssetContent, LocationContent};
use crate::relation::Reference;

/// The content of the beacon_info state event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3672.beacon_info", alias = "m.beacon" kind = State, state_key_type = OwnedUserId)]
pub struct BeaconInfoEventContent {

    /// is the same as an m.location description
    description: String,

    /// should be true when a user starts sharing location.
    live: bool,

    #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,

    /// is the length of time in milliseconds that the location will be live. So the location will stop being shared at m.ts + timeout milliseconds since the epoch.
    timeout: u64,

    /// The asset this message refers to.
    #[serde(default, rename = "org.matrix.msc3488.asset", skip_serializing_if = "ruma_common::serde::is_default")]
    pub asset: AssetContent,
}


/// The payload for a beacon event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3488.location", alias = "m.beacon", kind = MessageLike)]
pub struct BeaconEventRelationContent {

    /// Information about the poll start event this responds to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,

    /// The location info of the message.
    #[serde(rename = "org.matrix.msc3488.location")]
    pub location: LocationContent,

    #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}



impl BeaconInfoEventContent {
    /// Creates a new `BeaconInfoEventContent` with the given description, live, timeout and asset.
    pub fn start(description: String, timeout: u64, asset: AssetContent) -> Self {
        Self { description, live: true, ts: None, timeout, asset }
    }
}

impl BeaconEventRelationContent {
    /// Creates a new `BeaconEventRelationContent` with the given description, live, timeout and asset.
    pub fn new(relates_to: Reference, location: LocationContent, ts: Option<MilliSecondsSinceUnixEpoch>) -> Self {
        Self { relates_to, location, ts }
    }
}