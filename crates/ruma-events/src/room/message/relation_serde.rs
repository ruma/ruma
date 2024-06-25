use ruma_common::{serde::JsonObject, OwnedEventId};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value as JsonValue;

use super::{InReplyTo, Relation, RelationWithoutReplacement, Replacement, Thread};
use crate::relation::CustomRelation;

/// Deserialize an event's `relates_to` field.
///
/// Use it like this:
/// ```
/// # use serde::{Deserialize, Serialize};
/// use ruma_events::room::message::{deserialize_relation, MessageType, Relation};
///
/// #[derive(Deserialize, Serialize)]
/// struct MyEventContent {
///     #[serde(
///         flatten,
///         skip_serializing_if = "Option::is_none",
///         deserialize_with = "deserialize_relation"
///     )]
///     relates_to: Option<Relation<MessageType>>,
/// }
/// ```
pub fn deserialize_relation<'de, D, C>(deserializer: D) -> Result<Option<Relation<C>>, D::Error>
where
    D: Deserializer<'de>,
    C: Deserialize<'de>,
{
    let EventWithRelatesToDeHelper { relates_to, new_content } =
        EventWithRelatesToDeHelper::deserialize(deserializer)?;
    let Some(relates_to) = relates_to else {
        return Ok(None);
    };

    let RelatesToDeHelper { in_reply_to, relation } = relates_to;

    let rel = match relation {
        RelationDeHelper::Known(relation) => match relation {
            KnownRelationDeHelper::Replacement(ReplacementJsonRepr { event_id }) => {
                match new_content {
                    Some(new_content) => {
                        Relation::Replacement(Replacement { event_id, new_content })
                    }
                    None => return Err(de::Error::missing_field("m.new_content")),
                }
            }
            KnownRelationDeHelper::Thread(ThreadDeHelper { event_id, is_falling_back })
            | KnownRelationDeHelper::ThreadUnstable(ThreadUnstableDeHelper {
                event_id,
                is_falling_back,
            }) => Relation::Thread(Thread { event_id, in_reply_to, is_falling_back }),
        },
        RelationDeHelper::Unknown(c) => {
            if let Some(in_reply_to) = in_reply_to {
                Relation::Reply { in_reply_to }
            } else {
                Relation::_Custom(c)
            }
        }
    };

    Ok(Some(rel))
}

impl<C> Serialize for Relation<C>
where
    C: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (relates_to, new_content) = self.clone().into_parts();

        EventWithRelatesToSerHelper { relates_to, new_content }.serialize(serializer)
    }
}

#[derive(Deserialize)]
pub(crate) struct EventWithRelatesToDeHelper<C> {
    #[serde(rename = "m.relates_to")]
    relates_to: Option<RelatesToDeHelper>,

    #[serde(rename = "m.new_content")]
    new_content: Option<C>,
}

#[derive(Deserialize)]
pub(crate) struct RelatesToDeHelper {
    #[serde(rename = "m.in_reply_to")]
    in_reply_to: Option<InReplyTo>,

    #[serde(flatten)]
    relation: RelationDeHelper,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum RelationDeHelper {
    Known(KnownRelationDeHelper),
    Unknown(CustomRelation),
}

#[derive(Deserialize)]
#[serde(tag = "rel_type")]
pub(crate) enum KnownRelationDeHelper {
    #[serde(rename = "m.replace")]
    Replacement(ReplacementJsonRepr),

    #[serde(rename = "m.thread")]
    Thread(ThreadDeHelper),

    #[serde(rename = "io.element.thread")]
    ThreadUnstable(ThreadUnstableDeHelper),
}

/// A replacement relation without `m.new_content`.
#[derive(Deserialize, Serialize)]
pub(crate) struct ReplacementJsonRepr {
    event_id: OwnedEventId,
}

/// A thread relation without the reply fallback, with stable names.
#[derive(Deserialize)]
pub(crate) struct ThreadDeHelper {
    event_id: OwnedEventId,

    #[serde(default)]
    is_falling_back: bool,
}

/// A thread relation without the reply fallback, with unstable names.
#[derive(Deserialize)]
pub(crate) struct ThreadUnstableDeHelper {
    event_id: OwnedEventId,

    #[serde(rename = "io.element.show_reply", default)]
    is_falling_back: bool,
}

#[derive(Serialize)]
pub(super) struct EventWithRelatesToSerHelper<C> {
    #[serde(rename = "m.relates_to")]
    relates_to: RelationSerHelper,

    #[serde(rename = "m.new_content", skip_serializing_if = "Option::is_none")]
    new_content: Option<C>,
}

/// A relation, which associates new information to an existing event.
#[derive(Serialize)]
#[serde(tag = "rel_type")]
pub(super) enum RelationSerHelper {
    /// An event that replaces another event.
    #[serde(rename = "m.replace")]
    Replacement(ReplacementJsonRepr),

    /// An event that belongs to a thread, with stable names.
    #[serde(untagged)]
    Thread(Thread),

    /// An unknown relation type.
    #[serde(untagged)]
    Custom(CustomSerHelper),
}

impl<C> Relation<C> {
    fn into_parts(self) -> (RelationSerHelper, Option<C>) {
        match self {
            Relation::Replacement(Replacement { event_id, new_content }) => (
                RelationSerHelper::Replacement(ReplacementJsonRepr { event_id }),
                Some(new_content),
            ),
            Relation::Reply { in_reply_to } => {
                (RelationSerHelper::Custom(in_reply_to.into()), None)
            }
            Relation::Thread(t) => (RelationSerHelper::Thread(t), None),
            Relation::_Custom(c) => (RelationSerHelper::Custom(c.into()), None),
        }
    }

    pub(super) fn serialize_data(&self) -> JsonObject
    where
        C: Clone,
    {
        let (relates_to, _) = self.clone().into_parts();

        match serde_json::to_value(relates_to).expect("relation serialization to succeed") {
            JsonValue::Object(mut obj) => {
                obj.remove("rel_type");
                obj
            }
            _ => panic!("all relations must serialize to objects"),
        }
    }
}

#[derive(Default, Serialize)]
pub(super) struct CustomSerHelper {
    #[serde(rename = "m.in_reply_to", skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<InReplyTo>,

    #[serde(flatten, skip_serializing_if = "JsonObject::is_empty")]
    data: JsonObject,
}

impl From<InReplyTo> for CustomSerHelper {
    fn from(value: InReplyTo) -> Self {
        Self { in_reply_to: Some(value), data: JsonObject::new() }
    }
}

impl From<CustomRelation> for CustomSerHelper {
    fn from(CustomRelation(data): CustomRelation) -> Self {
        Self { in_reply_to: None, data }
    }
}

impl From<&RelationWithoutReplacement> for RelationSerHelper {
    fn from(value: &RelationWithoutReplacement) -> Self {
        match value.clone() {
            RelationWithoutReplacement::Reply { in_reply_to } => {
                RelationSerHelper::Custom(in_reply_to.into())
            }
            RelationWithoutReplacement::Thread(t) => RelationSerHelper::Thread(t),
            RelationWithoutReplacement::_Custom(c) => RelationSerHelper::Custom(c.into()),
        }
    }
}

impl Serialize for RelationWithoutReplacement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RelationSerHelper::from(self).serialize(serializer)
    }
}

impl RelationWithoutReplacement {
    pub(super) fn serialize_data(&self) -> JsonObject {
        let helper = RelationSerHelper::from(self);

        match serde_json::to_value(helper).expect("relation serialization to succeed") {
            JsonValue::Object(mut obj) => {
                obj.remove("rel_type");
                obj
            }
            _ => panic!("all relations must serialize to objects"),
        }
    }
}
