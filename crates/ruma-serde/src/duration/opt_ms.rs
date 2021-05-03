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

/// Serialize an Option<Duration>.
///
/// Will fail if integer is greater than the maximum integer that can be
/// unambiguously represented by an f64.
pub fn serialize<S>(opt_duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt_duration {
        Some(duration) => match UInt::try_from(duration.as_millis()) {
            Ok(uint) => uint.serialize(serializer),
            Err(err) => Err(S::Error::custom(err)),
        },
        None => serializer.serialize_none(),
    }
}

/// Deserializes an Option<Duration>.
///
/// Will fail if integer is greater than the maximum integer that can be
/// unambiguously represented by an f64.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<UInt>::deserialize(deserializer)?
        .map(|millis| Duration::from_millis(millis.into())))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    struct DurationTest {
        #[serde(with = "super", default, skip_serializing_if = "Option::is_none")]
        timeout: Option<Duration>,
    }

    #[test]
    fn deserialize_some() {
        let json = json!({ "timeout": 3000 });

        assert_eq!(
            serde_json::from_value::<DurationTest>(json).unwrap(),
            DurationTest { timeout: Some(Duration::from_millis(3000)) },
        );
    }

    #[test]
    fn deserialize_none_by_absence() {
        let json = json!({});

        assert_eq!(
            serde_json::from_value::<DurationTest>(json).unwrap(),
            DurationTest { timeout: None },
        );
    }

    #[test]
    fn deserialize_none_by_null() {
        let json = json!({ "timeout": null });

        assert_eq!(
            serde_json::from_value::<DurationTest>(json).unwrap(),
            DurationTest { timeout: None },
        );
    }

    #[test]
    fn serialize_some() {
        let request = DurationTest { timeout: Some(Duration::new(2, 0)) };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({ "timeout": 2000 }));
    }

    #[test]
    fn serialize_none() {
        let request = DurationTest { timeout: None };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({}));
    }
}
