//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use js_int::UInt;
use serde::Deserialize;

use super::Amplitude;

impl<'de> Deserialize<'de> for Amplitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uint = UInt::deserialize(deserializer)?;
        Ok(Self(uint.min(Self::MAX.into())))
    }
}
