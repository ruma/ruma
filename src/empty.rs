use std::fmt::{self, Formatter};

use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};

use crate::FromRaw;

/// A meaningless value that serializes to an empty JSON object.
///
/// This type is used in a few places where the Matrix specification requires an empty JSON object,
/// but it's wasteful to represent it as a `BTreeMap` in Rust code.
#[derive(Clone, Debug, PartialEq)]
pub struct Empty;

impl Serialize for Empty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmptyMapVisitor;

        impl<'de> Visitor<'de> for EmptyMapVisitor {
            type Value = Empty;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "an object/map")
            }

            fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                Ok(Empty)
            }
        }

        deserializer.deserialize_map(EmptyMapVisitor)
    }
}

impl FromRaw for Empty {
    type Raw = Self;

    fn from_raw(raw: Self) -> Self {
        raw
    }
}
