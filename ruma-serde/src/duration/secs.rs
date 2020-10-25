//! De-/serialization functions for `Option<std::time::Duration>` objects represented as
//! milliseconds.
//!
//! Delegates to `js_int::UInt` to ensure integer size is within bounds.

use std::{convert::TryFrom, time::Duration};

use js_int::UInt;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Error, Serialize, Serializer},
};

/// Serializes a Duration to an integer representing seconds.
///
/// Will fail if integer is greater than the maximum integer that can be
/// unambiguously represented by an f64.
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match UInt::try_from(duration.as_secs()) {
        Ok(uint) => uint.serialize(serializer),
        Err(err) => Err(S::Error::custom(err)),
    }
}

/// Deserializes an integer representing seconds into a Duration.
///
/// Will fail if integer is greater than the maximum integer that can be
/// unambiguously represented by an f64.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    UInt::deserialize(deserializer).map(|secs| Duration::from_secs(secs.into()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    struct DurationTest {
        #[serde(with = "super")]
        timeout: Duration,
    }

    #[test]
    fn deserialize() {
        let json = json!({ "timeout": 3 });

        assert_eq!(
            serde_json::from_value::<DurationTest>(json).unwrap(),
            DurationTest { timeout: Duration::from_secs(3) },
        );
    }

    #[test]
    fn serialize() {
        let test = DurationTest { timeout: Duration::from_millis(7000) };
        assert_eq!(serde_json::to_value(test).unwrap(), json!({ "timeout": 7 }));
    }
}
