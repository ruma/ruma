use std::fmt::{self, Formatter};

use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};

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

/// Serde serialization and deserialization functions that map a `Vec<T>` to a
/// `BTreeMap<T, Empty>`.
///
/// The Matrix spec sometimes specifies lists as hash maps so the list entries
/// can be expanded with attributes without breaking compatibility. As that
/// would be a breaking change for ruma's event types anyway, we convert them to
/// `Vec`s for simplicity, using this module.
///
/// To be used as `#[serde(with = "vec_as_map_of_empty")]`.
pub mod vec_as_map_of_empty {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::Empty;

    #[allow(clippy::ptr_arg)]
    pub fn serialize<S, T>(vec: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + Eq + Ord,
    {
        vec.iter().map(|v| (v, Empty)).collect::<BTreeMap<_, _>>().serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Eq + Ord,
    {
        BTreeMap::<T, Empty>::deserialize(deserializer)
            .map(|hashmap| hashmap.into_iter().map(|(k, _)| k).collect())
    }
}
