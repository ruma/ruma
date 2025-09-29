//! MSC4354 Sticky events types and utils.

use js_int::UInt;
use serde::{Deserialize, Serialize};

/// Sticky duration in milliseconds.
/// Valid values are the integer range 0-3600000 (1 hour)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct StickyDurationMs(u32);

impl StickyDurationMs {
    const MAX: u32 = 3_600_000;

    /// Creates a new `StickyDurationMs` if `v` is within `[0, 1h]`.
    ///
    /// Returns an error if `v` exceeds the maximum.
    pub fn new(v: u32) -> Result<Self, &'static str> {
        if v <= Self::MAX {
            Ok(Self(v))
        } else {
            Err("out of range [0, 3_600_000]")
        }
    }

    /// Creates a `DurationMs` by clamping `v` into `[0, 1h]`.
    pub fn new_wrapping<T: Into<u64>>(v: T) -> Self {
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

/// Message events can be annotated with a new top-level sticky object,
/// which MUST have a duration_ms, which is the number of milliseconds for the event to be
/// sticky.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StickyObject {
    // private, use `capped_duration_ms()`
    duration_ms: UInt,
}

impl StickyObject {
    /// Valid values are the integer range 0-3600000 (1 hour)
    pub fn clamped_duration_ms(&self) -> StickyDurationMs {
        StickyDurationMs::new_wrapping(self.duration_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::StickyDurationMs;

    #[test]
    fn new_wrapping_keeps_in_range_values() {
        let d = StickyDurationMs::new_wrapping(42u32);
        assert_eq!(d.0, 42);
    }

    #[test]
    fn new_wrapping_clamps_to_max_for_just_over_max() {
        let d = StickyDurationMs::new_wrapping(StickyDurationMs::MAX + 10_000);
        assert_eq!(d.0, StickyDurationMs::MAX);
    }

    #[test]
    fn new_wrapping_clamps_large_values_to_max() {
        let d = StickyDurationMs::new_wrapping(u64::MAX);
        assert_eq!(d.0, StickyDurationMs::MAX);
    }

    #[test]
    fn max_is_one_hour_in_ms() {
        assert_eq!(StickyDurationMs::MAX, 3_600_000);
    }
}
