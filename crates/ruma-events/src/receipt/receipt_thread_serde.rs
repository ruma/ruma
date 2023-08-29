use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ReceiptThread;

impl Serialize for ReceiptThread {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReceiptThread {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = ruma_common::serde::deserialize_cow_str(deserializer)?;
        Self::try_from(Some(s)).map_err(serde::de::Error::custom)
    }
}
