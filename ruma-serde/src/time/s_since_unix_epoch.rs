//! De-/serialization functions for `std::time::SystemTime` objects represented as seconds since
//! the UNIX epoch. Delegates to `js_int::UInt` to ensure integer size is within bounds.

use std::{
    convert::TryFrom,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use js_int::UInt;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Error, Serialize, Serializer},
};

/// Serialize a SystemTime.
///
/// Will fail if integer is greater than the maximum integer that can be unambiguously represented
/// by an f64.
pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // If this unwrap fails, the system this is executed is completely broken.
    let time_since_epoch = time.duration_since(UNIX_EPOCH).unwrap();
    match UInt::try_from(time_since_epoch.as_secs()) {
        Ok(uint) => uint.serialize(serializer),
        Err(err) => Err(S::Error::custom(err)),
    }
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
    Ok(UNIX_EPOCH + Duration::from_secs(secs.into()))
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
