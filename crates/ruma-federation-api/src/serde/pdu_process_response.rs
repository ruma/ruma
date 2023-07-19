use std::{collections::BTreeMap, fmt};

use ruma_common::OwnedEventId;
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

#[derive(Deserialize, Serialize)]
struct WrappedError {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub(crate) fn serialize<S>(
    response: &BTreeMap<OwnedEventId, Result<(), String>>,
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
                Err(error) => Some(error.clone()),
            },
        };
        map.serialize_entry(&key, &wrapped_error)?;
    }
    map.end()
}

#[allow(clippy::type_complexity)]
pub(crate) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<OwnedEventId, Result<(), String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct PduProcessResponseVisitor;

    impl<'de> Visitor<'de> for PduProcessResponseVisitor {
        type Value = BTreeMap<OwnedEventId, Result<(), String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("A map of EventIds to a map of optional errors")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = BTreeMap::new();

            while let Some((key, value)) = access.next_entry::<OwnedEventId, WrappedError>()? {
                let v = match value.error {
                    None => Ok(()),
                    Some(error) => Err(error),
                };
                map.insert(key, v);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(PduProcessResponseVisitor)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_common::{event_id, owned_event_id, OwnedEventId};
    use serde_json::{json, value::Serializer as JsonSerializer};

    use super::{deserialize, serialize};

    #[test]
    fn serialize_error() {
        let mut response: BTreeMap<OwnedEventId, Result<(), String>> = BTreeMap::new();
        response
            .insert(owned_event_id!("$someevent:matrix.org"), Err("Some processing error.".into()));

        let serialized = serialize(&response, JsonSerializer).unwrap();
        let json = json!({
            "$someevent:matrix.org": { "error": "Some processing error." }
        });
        assert_eq!(serialized, json);
    }

    #[test]
    fn serialize_ok() {
        let mut response: BTreeMap<OwnedEventId, Result<(), String>> = BTreeMap::new();
        response.insert(owned_event_id!("$someevent:matrix.org"), Ok(()));

        let serialized = serialize(&response, serde_json::value::Serializer).unwrap();
        let json = json!({
            "$someevent:matrix.org": {}
        });
        assert_eq!(serialized, json);
    }

    #[test]
    fn deserialize_error() {
        let json = json!({
            "$someevent:matrix.org": { "error": "Some processing error." }
        });

        let response = deserialize(json).unwrap();
        let event_id = event_id!("$someevent:matrix.org");

        let event_response = response.get(event_id).unwrap().clone().unwrap_err();
        assert_eq!(event_response, "Some processing error.");
    }

    #[test]
    fn deserialize_null_error_is_ok() {
        let json = json!({
            "$someevent:matrix.org": { "error": null }
        });

        let response = deserialize(json).unwrap();
        let event_id = event_id!("$someevent:matrix.org");

        response.get(event_id).unwrap().as_ref().unwrap();
    }

    #[test]
    fn deserialize_empty_error_is_err() {
        let json = json!({
            "$someevent:matrix.org": { "error": "" }
        });

        let response = deserialize(json).unwrap();
        let event_id = event_id!("$someevent:matrix.org");

        let event_response = response.get(event_id).unwrap().clone().unwrap_err();
        assert_eq!(event_response, "");
    }

    #[test]
    fn deserialize_ok() {
        let json = json!({
            "$someevent:matrix.org": {}
        });
        let response = deserialize(json).unwrap();
        response.get(event_id!("$someevent:matrix.org")).unwrap().as_ref().unwrap();
    }
}
