//! Types for extensible live location message events ([MSC3489]).
//!
//! [MSC3489]: https://github.com/matrix-org/matrix-spec-proposals/pull/3489


use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_macros::EventContent;
use crate::location::{AssetContent, LocationContent};
use crate::relation::Reference;

/// `BeaconInfoStateEventContent` is a struct that represents the content of a beacon_info state event.
/// It contains information about a live location sharing event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3672.beacon_info", alias = "m.beacon" kind = State, state_key_type = OwnedUserId)]
pub struct BeaconInfoStateEventContent {
    /// `description` is a string that is the same as an m.location description.
    description: String,

    /// `live` is a boolean that should be true when a user starts sharing location.
    pub live: bool,

    /// `ts` is an optional `MilliSecondsSinceUnixEpoch` that represents the timestamp of the event.
    #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,

    /// `timeout` is a u64 that represents the length of time in milliseconds that the location will be live.
    /// So the location will stop being shared at `m.ts + timeout` milliseconds since the epoch.
    timeout: u64,

    /// `asset` is an `AssetContent` that this message refers to.
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

impl BeaconInfoStateEventContent {
    /// Creates a new `BeaconInfoEventContent` with the given description, live, timeout and asset.
    pub fn start(description: String, timeout: u64, asset: AssetContent) -> Self {
        Self { description, live: true, ts: None, timeout, asset }
    }

    pub fn stop(&mut self) {
        self.live = false;
    }

    pub fn is_live(&self) -> bool {
        let now_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        self.live & self.ts.unwrap() + self.timeout < now_ts
    }
}

impl BeaconEventRelationContent {
    pub fn new(relates_to: Reference, location: LocationContent, ts: Option<MilliSecondsSinceUnixEpoch>) -> Self {
        Self { relates_to, location, ts }
    }
}
