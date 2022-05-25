//! `Deserialize` implementation for RoomMessageEventContent and MessageType.

use std::{convert::TryFrom, fmt};

use js_int::UInt;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use super::CallVersion;

impl<'de> Deserialize<'de> for CallVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CallVersionVisitor;

        impl<'de> Visitor<'de> for CallVersionVisitor {
            type Value = CallVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("string or uint")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value.into())
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let uint = UInt::try_from(value).map_err(de::Error::custom)?;
                Ok(uint.into())
            }
        }

        deserializer.deserialize_any(CallVersionVisitor)
    }
}

impl Serialize for CallVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Stable(version) => serializer.serialize_u64(version.to_owned().into()),
            Self::Namespaced(version) => serializer.serialize_str(version.as_ref()),
        }
    }
}
