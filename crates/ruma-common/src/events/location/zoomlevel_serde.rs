//! `Serialize` and `Deserialize` implementations for extensible events (MSC1767).

use js_int::UInt;
use serde::{de, Deserialize};

use super::{ZoomLevel, ZoomLevelError};

impl<'de> Deserialize<'de> for ZoomLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uint = UInt::deserialize(deserializer)?;
        if uint > Self::MAX.into() {
            Err(de::Error::custom(ZoomLevelError::TooHigh))
        } else {
            Ok(Self(uint))
        }
    }
}
