use ruma_common::{serde::from_raw_json_value, EventId};
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    NewUnstablePollStartEventContent, NewUnstablePollStartEventContentWithoutRelation,
    ReplacementUnstablePollStartEventContent, UnstablePollStartContentBlock,
    UnstablePollStartEventContent,
};
use crate::room::message::{deserialize_relation, Relation};

impl<'de> Deserialize<'de> for UnstablePollStartEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let mut deserializer = serde_json::Deserializer::from_str(json.get());
        let relates_to: Option<Relation<NewUnstablePollStartEventContentWithoutRelation>> =
            deserialize_relation(&mut deserializer).map_err(de::Error::custom)?;
        let UnstablePollStartEventContentDeHelper { poll_start, text } =
            from_raw_json_value(&json)?;

        let c = match relates_to {
            Some(Relation::Replacement(relates_to)) => {
                ReplacementUnstablePollStartEventContent { poll_start, text, relates_to }.into()
            }
            rel => {
                let poll_start = poll_start
                    .ok_or_else(|| de::Error::missing_field("org.matrix.msc3381.poll.start"))?;
                let relates_to = rel
                    .map(|r| r.try_into().expect("Relation::Replacement has already been handled"));
                NewUnstablePollStartEventContent { poll_start, text, relates_to }.into()
            }
        };

        Ok(c)
    }
}

#[derive(Debug, Deserialize)]
struct UnstablePollStartEventContentDeHelper {
    #[serde(rename = "org.matrix.msc3381.poll.start")]
    poll_start: Option<UnstablePollStartContentBlock>,

    #[serde(rename = "org.matrix.msc1767.text")]
    text: Option<String>,
}

impl Serialize for ReplacementUnstablePollStartEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = 2 + self.poll_start.is_some() as usize + self.text.is_some() as usize;

        let mut state =
            serializer.serialize_struct("ReplacementUnstablePollStartEventContent", len)?;

        if let Some(poll_start) = &self.poll_start {
            state.serialize_field("org.matrix.msc3381.poll.start", poll_start)?;
        }
        if let Some(text) = &self.text {
            state.serialize_field("org.matrix.msc1767.text", text)?;
        }

        state.serialize_field("m.new_content", &self.relates_to.new_content)?;
        state.serialize_field(
            "m.relates_to",
            &ReplacementRelatesTo { event_id: &self.relates_to.event_id },
        )?;

        state.end()
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "rel_type", rename = "m.replace")]
struct ReplacementRelatesTo<'a> {
    event_id: &'a EventId,
}
