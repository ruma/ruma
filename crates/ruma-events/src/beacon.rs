//! Types for the `org.matrix.msc3489.beacon` event, the unstable version of
//! `m.beacon`.

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedEventId};
use ruma_events::{location::LocationContent, relation::Reference};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// `BeaconEventContent` is a struct that represents the content of a beacon
/// event. It contains information about a user location in reference to a beacon_info state event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3672.beacon", alias = "m.beacon", kind = MessageLike)]
pub struct BeaconEventContent {
    /// The beacon_info event id this relates to.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,

    /// The location of the beacon.
    #[serde(rename = "org.matrix.msc3488.location")]
    pub location: LocationContent,

    /// The timestamp of the event.
    #[serde(rename = "org.matrix.msc3488.ts")]
    pub ts: MilliSecondsSinceUnixEpoch,
}

impl BeaconEventContent {
    /// Create a new `BeaconEventContent` with the given beacon_info_event_id and geo_uri
    pub fn new(
        beacon_info_event_id: OwnedEventId,
        geo_uri: String,
        ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        Self {
            relates_to: Reference::new(beacon_info_event_id),
            location: LocationContent::new(geo_uri),
            ts: ts.unwrap_or_else(MilliSecondsSinceUnixEpoch::now),
        }
    }
}
