use std::{collections::BTreeMap, fmt};

use ruma_identifiers::EventId;
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

pub fn serialize<S>(
    response: BTreeMap<EventId, Result<(), String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(response.len()))?;
    for (key, value) in response {
        let wrapped_error = WrappedError {
            error: match value {
                Ok(_) => None,
                Err(error) => Some(error),
            },
        };
        map.serialize_entry(&key, &wrapped_error)?;
    }
    map.end()
}

pub fn deserialize<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<EventId, Result<(), String>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(PduProcessResponseVisitor {})
}

#[derive(Deserialize, Serialize)]
struct WrappedError {
    error: Option<String>,
}

struct PduProcessResponseVisitor;

impl<'de> Visitor<'de> for PduProcessResponseVisitor {
    type Value = BTreeMap<EventId, Result<(), String>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A map of EventIds to a map of optional errors")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = BTreeMap::new();

        while let Some((key, value)) = access.next_entry::<EventId, WrappedError>()? {
            let v = match value.error {
                None => Ok(()),
                Some(error) => Err(error),
            };
            map.insert(key, v);
        }
        Ok(map)
    }
}
