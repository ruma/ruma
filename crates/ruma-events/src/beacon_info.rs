//! Types for extensible live location message events ([MSC3489]).
//!
//! [MSC3489]: https://github.com/matrix-org/matrix-spec-proposals/pull/3489

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{location::LocationContent, relation::Reference};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3672.beacon_info", alias = "m.beacon_info", kind = MessageLike)]
pub struct BeaconInfoEventContent {
    /// Information about the beacon state event this relates to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,

    /// The location info of the message.
    #[serde(rename = "org.matrix.msc3488.location")]
    pub location: LocationContent,

    /// optional `MilliSecondsSinceUnixEpoch` that represents the timestamp of the event.
    #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl BeaconInfoEventContent {
    /// Create a new `BeaconInfoEventContent` with the given relates_to, location and timestamp.
    pub fn new(
        relates_to: Reference,
        location: LocationContent,
        ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        Self { relates_to, location, ts }
    }
}
