use std::collections::BTreeMap;

use ruma_common::serde::from_raw_json_value;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};
use serde_json::value::RawValue as RawJsonValue;

use super::v3::{CustomResultGroupMap, GroupingKey, ResultGroupMap, ResultGroupMapsByGroupingKey};

impl Serialize for ResultGroupMapsByGroupingKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(Some(self.len()))?;

        for map in self.values() {
            match map {
                ResultGroupMap::RoomId(map) => s.serialize_entry(&GroupingKey::RoomId, map)?,
                ResultGroupMap::Sender(map) => s.serialize_entry(&GroupingKey::Sender, map)?,
                ResultGroupMap::_Custom(CustomResultGroupMap { grouping_key, map }) => {
                    s.serialize_entry(grouping_key, map)?;
                }
            }
        }

        s.end()
    }
}

impl<'de> Deserialize<'de> for ResultGroupMapsByGroupingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map_by_key = BTreeMap::<GroupingKey, Box<RawJsonValue>>::deserialize(deserializer)?;

        map_by_key
            .into_iter()
            .map(|(grouping_key, map)| {
                Ok(match grouping_key {
                    GroupingKey::RoomId => ResultGroupMap::RoomId(from_raw_json_value(&map)?),
                    GroupingKey::Sender => ResultGroupMap::Sender(from_raw_json_value(&map)?),
                    GroupingKey::_Custom(s) => ResultGroupMap::_Custom(CustomResultGroupMap {
                        grouping_key: s.0.into(),
                        map: from_raw_json_value(&map)?,
                    }),
                })
            })
            .collect()
    }
}
