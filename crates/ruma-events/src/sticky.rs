//! Types for the sticky events event defined in [MSC4354].
//!
//! [MSC4354]: https://github.com/matrix-org/matrix-spec-proposals/pull/4354

use std::fmt::Formatter;

use serde::{Deserialize, Serialize, de::Error};

/// Sticky duration in milliseconds.
///
/// Valid values are the integer range 0-3600000 (1 hour).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct StickyDurationMs(u32);

/// New event top-level sticky object.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StickyObject {
    /// The number of milliseconds for the event to be sticky.
    pub duration_ms: StickyDurationMs,
}

impl StickyDurationMs {
    /// The maximum possible sticky duration in millis (1 hour).
    pub const MAX: u32 = 3_600_000;

    /// Creates a `StickyDurationMs` by clamping `v` into `[0, 1h]`.
    pub fn new_clamped<T: Into<u64>>(v: T) -> Self {
        let v = v.into();
        let clamped = v.min(Self::MAX as u64) as u32;
        Self(clamped)
    }

    /// Get the value as a `u32`.
    pub fn get(self) -> u32 {
        self.into()
    }
}

impl From<StickyDurationMs> for u32 {
    fn from(d: StickyDurationMs) -> Self {
        d.0
    }
}

impl<'de> Deserialize<'de> for StickyDurationMs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StickyDurationMsVisitor;

        impl<'de> serde::de::Visitor<'de> for StickyDurationMsVisitor {
            type Value = StickyDurationMs;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "an integer in the range 0-3600000")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v < 0 {
                    Err(E::invalid_value(serde::de::Unexpected::Signed(v), &self))
                } else {
                    self.visit_u64(v as u64)
                }
            }

            // serde json deserialize json numbers as u64 by default
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v > StickyDurationMs::MAX as u64 {
                    Err(E::invalid_value(serde::de::Unexpected::Unsigned(v), &self))
                } else {
                    Ok(StickyDurationMs(v as u32))
                }
            }
        }

        deserializer.deserialize_any(StickyDurationMsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::sticky::{StickyDurationMs, StickyObject};

    #[test]
    fn deserialize_sticky_ok() {
        let raw = "78000";
        let sticky_duration: StickyDurationMs = serde_json::from_str(raw).unwrap();
        assert_eq!(sticky_duration.get(), 78_000_u32);
    }

    #[test]
    fn serialize_sticky() {
        let sticky = StickyDurationMs::new_clamped(60_000_u32);
        let ser = serde_json::to_string(&sticky).unwrap();
        assert_eq!(ser, "60000");
    }

    #[test]
    fn deserialize_sticky_oob() {
        let raw = "3600001";
        let res: Result<StickyDurationMs, _> = serde_json::from_str(raw);
        assert!(res.is_err());

        let err = res.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains(
            "invalid value: integer `3600001`, expected an integer in the range 0-3600000"
        ));
    }

    #[test]
    fn deserialize_sticky_float_err() {
        let raw = "3000.0";
        let res: Result<StickyDurationMs, _> = serde_json::from_str(raw);
        assert!(res.is_err());

        let err = res.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains(
            "invalid type: floating point `3000.0`, expected an integer in the range 0-3600000"
        ));
    }

    #[test]
    fn deserialize_sticky_neg_err() {
        let raw = "-1";
        let res: Result<StickyDurationMs, _> = serde_json::from_str(raw);
        assert!(res.is_err());

        let err = res.unwrap_err();
        let err_msg = err.to_string();
        println!("{}", err_msg);
        assert!(
            err_msg.contains(
                "invalid value: integer `-1`, expected an integer in the range 0-3600000"
            )
        );
    }

    #[test]
    fn deserialize_sticky_object_ok() {
        let ser = r#"{"duration_ms":78000}"#;
        let sticky_object: StickyObject = serde_json::from_str(ser).unwrap();
        assert_eq!(sticky_object.duration_ms.get(), 78_000_u32);
    }

    #[test]
    fn deserialize_sticky_object_err() {
        let raw = r#"{"duration_ms":3600001}"#;
        let res: Result<StickyObject, _> = serde_json::from_str(raw);
        assert!(res.is_err());
    }
}
