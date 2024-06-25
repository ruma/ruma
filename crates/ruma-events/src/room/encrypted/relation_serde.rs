use ruma_common::{
    serde::{from_raw_json_value, JsonObject},
    OwnedEventId,
};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

use super::{InReplyTo, Relation, Thread};

impl<'de> Deserialize<'de> for Relation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let RelationDeHelper { in_reply_to, rel_type } = from_raw_json_value(&json)?;

        let rel = match (in_reply_to, rel_type.as_deref()) {
            (_, Some("m.thread")) => Relation::Thread(from_raw_json_value(&json)?),
            (in_reply_to, Some("io.element.thread")) => {
                let ThreadUnstableDeHelper { event_id, is_falling_back } =
                    from_raw_json_value(&json)?;
                Relation::Thread(Thread { event_id, in_reply_to, is_falling_back })
            }
            (_, Some("m.annotation")) => Relation::Annotation(from_raw_json_value(&json)?),
            (_, Some("m.reference")) => Relation::Reference(from_raw_json_value(&json)?),
            (_, Some("m.replace")) => Relation::Replacement(from_raw_json_value(&json)?),
            (Some(in_reply_to), _) => Relation::Reply { in_reply_to },
            _ => Relation::_Custom(from_raw_json_value(&json)?),
        };

        Ok(rel)
    }
}

#[derive(Default, Deserialize)]
struct RelationDeHelper {
    #[serde(rename = "m.in_reply_to")]
    in_reply_to: Option<InReplyTo>,

    rel_type: Option<String>,
}

/// A thread relation without the reply fallback, with unstable names.
#[derive(Clone, Deserialize)]
struct ThreadUnstableDeHelper {
    event_id: OwnedEventId,

    #[serde(rename = "io.element.show_reply", default)]
    is_falling_back: bool,
}

impl Relation {
    pub(super) fn serialize_data(&self) -> JsonObject {
        match serde_json::to_value(self).expect("relation serialization to succeed") {
            JsonValue::Object(mut obj) => {
                obj.remove("rel_type");
                obj
            }
            _ => panic!("all relations must serialize to objects"),
        }
    }
}

impl Serialize for Relation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Relation::Reply { in_reply_to } => {
                let mut st = serializer.serialize_struct("Relation", 1)?;
                st.serialize_field("m.in_reply_to", in_reply_to)?;
                st.end()
            }
            Relation::Replacement(data) => data.serialize(serializer),
            Relation::Reference(data) => data.serialize(serializer),
            Relation::Annotation(data) => data.serialize(serializer),
            Relation::Thread(data) => data.serialize(serializer),
            Relation::_Custom(c) => c.serialize(serializer),
        }
    }
}
