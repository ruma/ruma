//! De-/serialization functions to and from single element sequences.

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// Serialize the given value as a list of just that value.
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    [value].serialize(serializer)
}

/// Deserialize a list of one item and return that item.
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    <[_; 1]>::deserialize(deserializer).map(|[first]| first)
}
