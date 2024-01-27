#![cfg(feature = "unstable-msc3489")]

use ruma_events::beacon::BeaconStateEventContent;

#[test]
fn beacon_starts_not_live() {
    let timeout = std::time::Duration::from_secs(60);

    let beacon = BeaconStateEventContent::new(None, timeout);

    assert_eq!(beacon.live, false);
}
