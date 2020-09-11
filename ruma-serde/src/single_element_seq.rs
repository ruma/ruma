//! De-/serialization functions to and from single element sequences.

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

pub fn serialize<T, S>(element: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    [element].serialize(serializer)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    <[_; 1]>::deserialize(deserializer).map(|[first]| first)
}
