use serde::{
    de::{self, Deserialize, Deserializer, Unexpected},
    Serialize, Serializer,
};

/// A type that can be used as the `state_key` for event types where that field is always empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::exhaustive_structs)]
pub struct EmptyStateKey;

impl AsRef<str> for EmptyStateKey {
    fn as_ref(&self) -> &str {
        ""
    }
}

impl<'de> Deserialize<'de> for EmptyStateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = crate::serde::deserialize_cow_str(deserializer)?;
        if s.is_empty() {
            Ok(EmptyStateKey)
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(&s), &"an empty string"))
        }
    }
}

impl Serialize for EmptyStateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("")
    }
}
