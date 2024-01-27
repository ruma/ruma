#![cfg(feature = "unstable-msc3489")]

use js_int::UInt;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::beacon::BeaconStateEventContent;

#[test]
fn beacon_starts_not_live() {
    let now = MilliSecondsSinceUnixEpoch::now();

    // Calculate 5 minutes in milliseconds
    let five_minutes_in_millis = 5 * 60 * 1000;

    // Add 5 minutes to the current time
    let timeout = MilliSecondsSinceUnixEpoch(now.get() + UInt::from(five_minutes_in_millis));

    let beacon = BeaconStateEventContent::new(None, timeout);

    assert_eq!(beacon.live, false);
}
