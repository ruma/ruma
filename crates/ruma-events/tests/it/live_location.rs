#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};
    use js_int::UInt;
    use ruma_common::MilliSecondsSinceUnixEpoch;
    use ruma_events::live_location::BeaconInfoStateEventContent;
    use ruma_events::location::AssetContent;

    #[test]
    fn beacon_starts_live() {
        let beacon = BeaconInfoStateEventContent::start("Test Beacon".to_string(), 5000, AssetContent::default());
        assert!(beacon.live);
    }
    #[test]
    fn beacon_stops_live() {
        let mut beacon = BeaconInfoStateEventContent::start("Test Beacon".to_string(), 5000, AssetContent::default());
        beacon.stop();
        assert!(!beacon.live);
    }
    #[test]
    fn beacon_is_live_within_timeout() {
        let mut beacon = BeaconInfoStateEventContent::start("Test Beacon".to_string(), 5000, AssetContent::default());
        beacon.ts = Some(MilliSecondsSinceUnixEpoch(UInt::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - 1000).unwrap()));
        assert!(beacon.is_live());
    }

    #[test]
    fn beacon_is_not_live_past_timeout() {
        let mut beacon = BeaconInfoStateEventContent::start("Test Beacon".to_string(), 5000, AssetContent::default());
        beacon.ts = Some(MilliSecondsSinceUnixEpoch(UInt::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - 6000).unwrap()));
        assert!(!beacon.is_live());
    }

    #[test]
    fn beacon_is_not_live_when_ts_is_none() {
        let beacon = BeaconInfoStateEventContent::start("Test Beacon".to_string(), 5000, AssetContent::default());
        assert!(!beacon.is_live());
    }
}