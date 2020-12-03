use serde::{Deserialize, Deserializer};

/// Used with `#[serde(deserialize_with)]`. Like `#[serde(default)]`, but also applies
/// if the input is a unit value (`null`).
pub fn default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

/// Used with `#[serde(deserialize_with)]`. Like `#[serde(default = "get_true")]`, but also applies
/// if the input is a unit value (`null`).
pub fn default_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or(true))
}
