use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::RoomTypeFilter;

impl Serialize for RoomTypeFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RoomTypeFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<Cow<'_, str>>::deserialize(deserializer)?;
        Ok(s.into())
    }
}
