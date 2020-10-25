//! De-/serialization functions for `Option<std::time::SystemTime>` objects represented as
//! seconds since the UNIX epoch. Delegates to `js_int::UInt` to ensure integer size is within
//! bounds.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use js_int::UInt;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// Serialize an `Option<SystemTime>`.
///
/// Will fail if integer is greater than the maximum integer that can be unambiguously represented
/// by an f64.
pub fn serialize<S>(opt_time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt_time {
        Some(time) => super::s_since_unix_epoch::serialize(time, serializer),
        None => Option::<UInt>::serialize(&None, serializer),
    }
}

/// Deserializes an `Option<SystemTime>`.
///
/// Will fail if integer is greater than the maximum integer that can be unambiguously represented
/// by an f64.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<UInt>::deserialize(deserializer)?
        .map(|secs| UNIX_EPOCH + Duration::from_secs(secs.into())))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    struct SystemTimeTest {
        #[serde(with = "super", default, skip_serializing_if = "Option::is_none")]
        timestamp: Option<SystemTime>,
    }

    #[test]
    fn deserialize_some() {
        let json = json!({ "timestamp": 3000 });

        assert_eq!(
            serde_json::from_value::<SystemTimeTest>(json).unwrap(),
            SystemTimeTest { timestamp: Some(UNIX_EPOCH + Duration::from_secs(3000)) },
        );
    }

    #[test]
    fn deserialize_none_by_absence() {
        let json = json!({});

        assert_eq!(
            serde_json::from_value::<SystemTimeTest>(json).unwrap(),
            SystemTimeTest { timestamp: None },
        );
    }

    #[test]
    fn deserialize_none_by_null() {
        let json = json!({ "timestamp": null });

        assert_eq!(
            serde_json::from_value::<SystemTimeTest>(json).unwrap(),
            SystemTimeTest { timestamp: None },
        );
    }

    #[test]
    fn serialize_some() {
        let request = SystemTimeTest { timestamp: Some(UNIX_EPOCH + Duration::new(2000, 0)) };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({ "timestamp": 2000 }));
    }

    #[test]
    fn serialize_none() {
        let request = SystemTimeTest { timestamp: None };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({}));
    }
}
