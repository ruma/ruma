#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::EventId;
use serde::{ser::SerializeStruct as _, Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "unstable-pre-spec")]
use super::Replacement;
use super::{InReplyTo, Relation};
#[cfg(feature = "unstable-pre-spec")]
use crate::room::message::MessageEventContent;

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Relation>, D::Error>
where
    D: Deserializer<'de>,
{
    fn convert_relation(ev: EventWithRelatesToJsonRepr) -> Option<Relation> {
        if let Some(in_reply_to) = ev.relates_to.in_reply_to {
            return Some(Relation::Reply { in_reply_to });
        }

        #[cfg(feature = "unstable-pre-spec")]
        if let Some(relation) = ev.relates_to.relation {
            let relation = match relation {
                RelationJsonRepr::Replacement(ReplacementJsonRepr { event_id }) => {
                    let new_content = ev.new_content?;
                    Relation::Replacement(Replacement { event_id, new_content })
                }
                // FIXME: Maybe we should log this, though at this point we don't even have access
                // to the rel_type of the unknown relation.
                RelationJsonRepr::Unknown => return None,
            };

            return Some(relation);
        }

        None
    }

    EventWithRelatesToJsonRepr::deserialize(deserializer).map(convert_relation)
}

pub fn serialize<S>(relation: &Option<Relation>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let relation = match relation {
        Some(rel) => rel,
        // FIXME: If this crate ends up depending on tracing, emit a warning here.
        // This code path should not be reachable due to the skip_serializing_if serde attribute
        // that should be applied together with `with = "relation_serde"`.
        None => return serializer.serialize_struct("NoRelation", 0)?.end(),
    };

    let json_repr = match relation {
        Relation::Reply { in_reply_to } => EventWithRelatesToJsonRepr::new(RelatesToJsonRepr {
            in_reply_to: Some(in_reply_to.clone()),
            ..Default::default()
        }),
        #[cfg(feature = "unstable-pre-spec")]
        Relation::Replacement(Replacement { event_id, new_content }) => {
            EventWithRelatesToJsonRepr {
                relates_to: RelatesToJsonRepr {
                    relation: Some(RelationJsonRepr::Replacement(ReplacementJsonRepr {
                        event_id: event_id.clone(),
                    })),
                    ..Default::default()
                },
                new_content: Some(new_content.clone()),
            }
        }
    };

    json_repr.serialize(serializer)
}

#[derive(Deserialize, Serialize)]
struct EventWithRelatesToJsonRepr {
    #[serde(rename = "m.relates_to", default, skip_serializing_if = "RelatesToJsonRepr::is_empty")]
    relates_to: RelatesToJsonRepr,

    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.new_content", skip_serializing_if = "Option::is_none")]
    new_content: Option<Box<MessageEventContent>>,
}

impl EventWithRelatesToJsonRepr {
    fn new(relates_to: RelatesToJsonRepr) -> Self {
        Self {
            relates_to,
            #[cfg(feature = "unstable-pre-spec")]
            new_content: None,
        }
    }
}

/// Enum modeling the different ways relationships can be expressed in a `m.relates_to` field of an
/// event.
#[derive(Default, Deserialize, Serialize)]
struct RelatesToJsonRepr {
    #[serde(rename = "m.in_reply_to", skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<InReplyTo>,

    #[cfg(feature = "unstable-pre-spec")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    relation: Option<RelationJsonRepr>,
}

impl RelatesToJsonRepr {
    fn is_empty(&self) -> bool {
        #[cfg(not(feature = "unstable-pre-spec"))]
        {
            self.in_reply_to.is_none()
        }

        #[cfg(feature = "unstable-pre-spec")]
        {
            self.in_reply_to.is_none() && self.relation.is_none()
        }
    }
}

/// A relation, which associates new information to an existing event.
#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
#[serde(tag = "rel_type")]
enum RelationJsonRepr {
    /// An event that replaces another event.
    #[serde(rename = "m.replace")]
    Replacement(ReplacementJsonRepr),

    /// An unknown relation type.
    ///
    /// Not available in the public API, but exists here so deserialization
    /// doesn't fail with new / custom `rel_type`s.
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
struct ReplacementJsonRepr {
    event_id: EventId,
}
