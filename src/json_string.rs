//! De-/serialization functions to and from json strings, allows the type to be used as a query string.

use serde::{
    de::{Deserialize, DeserializeOwned, Deserializer, Error as _},
    ser::{Error as _, Serialize, Serializer},
};

pub fn serialize<T, S>(filter: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    let json = serde_json::to_string(&filter).map_err(S::Error::custom)?;
    serializer.serialize_str(&json)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(D::Error::custom)
}
