//! De-/serialization functions for `std::time::SystemTime` objects represented as seconds since
//! the UNIX epoch. Delegates to `js_int::UInt` to ensure integer size is within bounds.

use std::{
    convert::TryInto,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use js_int::UInt;
use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{self, Serialize, Serializer},
};

/// Serialize a SystemTime.
///
/// Will fail if integer is greater than the maximum integer that can be unambiguously represented
/// by an f64.
pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let time_since_epoch = time.duration_since(UNIX_EPOCH).map_err(ser::Error::custom)?;
    let uint: UInt = time_since_epoch.as_secs().try_into().map_err(ser::Error::custom)?;

    uint.serialize(serializer)
}

/// Deserializes a SystemTime.
///
/// Will fail if integer is greater than the maximum integer that can be unambiguously represented
/// by an f64.
pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = UInt::deserialize(deserializer)?;
    UNIX_EPOCH
        .checked_add(Duration::from_secs(secs.into()))
        .ok_or_else(|| de::Error::custom("input too large for SystemTime"))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    struct SystemTimeTest {
        #[serde(with = "super")]
        timestamp: SystemTime,
    }

    #[test]
    fn deserialize() {
        let json = json!({ "timestamp": 3000 });

        assert_eq!(
            serde_json::from_value::<SystemTimeTest>(json).unwrap(),
            SystemTimeTest { timestamp: UNIX_EPOCH + Duration::from_secs(3000) },
        );
    }

    #[test]
    fn serialize() {
        let request = SystemTimeTest { timestamp: UNIX_EPOCH + Duration::new(2000, 0) };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({ "timestamp": 2000 }));
    }
}
