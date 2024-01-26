#![cfg(feature = "unstable-msc3489")]

use std::time::{SystemTime, UNIX_EPOCH};

use js_int::UInt;

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::live_location::BeaconInfoStateEventContent;
use ruma_events::location::AssetContent;

#[test]
fn beacon_starts_live() {
    // Get the current time in milliseconds since the Unix epoch
    let now = MilliSecondsSinceUnixEpoch::now();

    // Calculate 5 minutes in milliseconds
    let five_minutes_in_millis = 5 * 60 * 1000;

    // Add 5 minutes to the current time
    let timeout = MilliSecondsSinceUnixEpoch(now.get() + UInt::from(five_minutes_in_millis));

    let mut beacon = BeaconInfoStateEventContent::new("Test Beacon".to_string(), timeout, AssetContent::default());
    beacon.start();
    assert!(beacon.live);
}
#[test]
fn beacon_stops_live() {
    let now = MilliSecondsSinceUnixEpoch::now();

    // Calculate 5 minutes in milliseconds
    let five_minutes_in_millis = 5 * 60 * 1000;

    // Add 5 minutes to the current time
    let timeout = MilliSecondsSinceUnixEpoch(now.get() + UInt::from(five_minutes_in_millis));
    let mut beacon = BeaconInfoStateEventContent::new("Test Beacon".to_string(), timeout, AssetContent::default());
    beacon.stop();
    assert!(!beacon.live);
}
#[test]
fn beacon_is_live_within_timeout() {
    let now = MilliSecondsSinceUnixEpoch::now();

    // Calculate 5 minutes in milliseconds
    let five_minutes_in_millis = 5 * 60 * 1000;

    // Add 5 minutes to the current time
    let timeout = MilliSecondsSinceUnixEpoch(now.get() + UInt::from(five_minutes_in_millis));

    let mut beacon = BeaconInfoStateEventContent::new("Test Beacon".to_string(), timeout, AssetContent::default());
    beacon.ts = Some(MilliSecondsSinceUnixEpoch(UInt::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - 1000).unwrap()));
    assert!(beacon.is_live());
}

#[test]
fn beacon_is_not_live_past_timeout() {
    let now = MilliSecondsSinceUnixEpoch::now();

    // Calculate 5 minutes in milliseconds
    let five_minutes_in_millis = 5 * 60 * 1000;

    // Add 5 minutes to the current time
    let timeout = MilliSecondsSinceUnixEpoch(now.get() + UInt::from(five_minutes_in_millis));

    let mut beacon = BeaconInfoStateEventContent::new("Test Beacon".to_string(), timeout, AssetContent::default());
    beacon.ts = Some(MilliSecondsSinceUnixEpoch(UInt::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - 6000).unwrap()));
    assert!(!beacon.is_live());
}

#[test]
fn beacon_is_not_live_when_ts_is_none() {
    let now = MilliSecondsSinceUnixEpoch::now();

    // Calculate 5 minutes in milliseconds
    let five_minutes_in_millis = 5 * 60 * 1000;

    // Add 5 minutes to the current time
    let timeout = MilliSecondsSinceUnixEpoch(now.get() + UInt::from(five_minutes_in_millis));
    let mut beacon = BeaconInfoStateEventContent::new("Test Beacon".to_string(), timeout, AssetContent::default());
    beacon.start();
    assert!(!beacon.is_live());
}