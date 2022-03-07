#[cfg(feature = "unstable-msc2676")]
use ruma_identifiers::EventId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "unstable-msc2676")]
use super::Replacement;
use super::{InReplyTo, Relation};
#[cfg(feature = "unstable-msc2676")]
use crate::room::message::RoomMessageEventContent;

impl<'de> Deserialize<'de> for Relation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ev = EventWithRelatesToJsonRepr::deserialize(deserializer)?;

        if let Some(in_reply_to) = ev.relates_to.in_reply_to {
            return Ok(Relation::Reply { in_reply_to });
        }

        #[cfg(feature = "unstable-msc2676")]
        if let Some(relation) = ev.relates_to.relation {
            return Ok(match relation {
                RelationJsonRepr::Replacement(ReplacementJsonRepr { event_id }) => {
                    let new_content = ev
                        .new_content
                        .ok_or_else(|| serde::de::Error::missing_field("m.new_content"))?;
                    Relation::Replacement(Replacement { event_id, new_content })
                }
                // FIXME: Maybe we should log this, though at this point we don't even have
                // access to the rel_type of the unknown relation.
                RelationJsonRepr::Unknown => Relation::_Custom,
            });
        }

        Ok(Relation::_Custom)
    }
}

impl Serialize for Relation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[allow(clippy::needless_update)]
        let json_repr = match self {
            Relation::Reply { in_reply_to } => EventWithRelatesToJsonRepr::new(RelatesToJsonRepr {
                in_reply_to: Some(in_reply_to.clone()),
                ..Default::default()
            }),
            #[cfg(feature = "unstable-msc2676")]
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
            Relation::_Custom => EventWithRelatesToJsonRepr::default(),
        };

        json_repr.serialize(serializer)
    }
}

#[derive(Default, Deserialize, Serialize)]
struct EventWithRelatesToJsonRepr {
    #[serde(rename = "m.relates_to", default, skip_serializing_if = "RelatesToJsonRepr::is_empty")]
    relates_to: RelatesToJsonRepr,

    #[cfg(feature = "unstable-msc2676")]
    #[serde(rename = "m.new_content", skip_serializing_if = "Option::is_none")]
    new_content: Option<Box<RoomMessageEventContent>>,
}

impl EventWithRelatesToJsonRepr {
    fn new(relates_to: RelatesToJsonRepr) -> Self {
        Self {
            relates_to,
            #[cfg(feature = "unstable-msc2676")]
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

    #[cfg(feature = "unstable-msc2676")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    relation: Option<RelationJsonRepr>,
}

impl RelatesToJsonRepr {
    fn is_empty(&self) -> bool {
        #[cfg(not(feature = "unstable-msc2676"))]
        {
            self.in_reply_to.is_none()
        }

        #[cfg(feature = "unstable-msc2676")]
        {
            self.in_reply_to.is_none() && self.relation.is_none()
        }
    }
}

/// A relation, which associates new information to an existing event.
#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2676")]
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
#[cfg(feature = "unstable-msc2676")]
struct ReplacementJsonRepr {
    event_id: Box<EventId>,
}
