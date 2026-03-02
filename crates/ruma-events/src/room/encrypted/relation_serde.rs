use ruma_common::{
    OwnedEventId,
    serde::{JsonObject, from_raw_json_value},
};
use serde::{Deserialize, Deserializer};
use serde_json::{Value as JsonValue, value::RawValue as RawJsonValue};

use super::{InReplyTo, Relation, Reply, Thread};

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
            (Some(in_reply_to), _) => Relation::Reply(Reply { in_reply_to }),
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
