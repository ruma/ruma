//! Types for the `org.matrix.msc3489.beacon_info` state event, the unstable version of
//! `m.beacon_info`.

use std::time::Duration;

use js_int::UInt;
use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedUserId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::location::AssetContent;

/// `BeaconInfoEventContent` is a struct that represents the content of a beacon_info state
/// event. It contains information about a live location share.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3672.beacon_info", alias = "m.beacon_info", kind = State, state_key_type = OwnedUserId)]
pub struct BeaconInfoEventContent {
    /// The description of the location.
    ///
    /// It should be used to label the location on a map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// `live` is a boolean that should be true when a user starts sharing location.
    pub live: bool,

    /// The time when location sharing started, in milliseconds since the epoch.
    #[serde(rename = "org.matrix.msc3488.ts")]
    pub ts: MilliSecondsSinceUnixEpoch,

    /// `timeout` represents the length of time in milliseconds that the location
    /// will be live. So the location will stop being shared at `m.ts + timeout` milliseconds
    /// since the epoch.
    #[serde(default, with = "ruma_common::serde::duration::ms")]
    pub timeout: Duration,

    /// `asset` is an [`AssetContent`] that this message refers to.
    #[serde(default, rename = "org.matrix.msc3488.asset")]
    pub asset: AssetContent,
}

impl BeaconInfoEventContent {
    /// Creates a new `BeaconInfoEventContent` with the given description, live, timeout and asset.
    pub fn new(
        description: Option<String>,
        timeout: Duration,
        live: bool,
        ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        Self {
            description,
            live,
            ts: ts.unwrap_or_else(MilliSecondsSinceUnixEpoch::now),
            timeout,
            asset: Default::default(),
        }
    }

    /// starts the beacon being live.
    pub fn start(&mut self) {
        self.live = true;
    }

    /// Stops the beacon from being live.
    pub fn stop(&mut self) {
        self.live = false;
    }

    /// Checks if the beacon is currently live.
    ///
    /// This method calculates the current time and compares it with the beacon's start time plus
    /// its timeout. If the beacon is not live or the current time is greater than the beacon's
    /// start time plus its timeout, it returns false, indicating that the beacon is not live.
    /// Otherwise, it returns true.
    pub fn is_live(&self) -> bool {
        self.live
            && self.ts.get() + UInt::try_from(self.timeout.as_millis()).unwrap()
                > MilliSecondsSinceUnixEpoch::now().get()
    }
}
