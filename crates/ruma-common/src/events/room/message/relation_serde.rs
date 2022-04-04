use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "unstable-msc2676")]
use super::Replacement;
#[cfg(feature = "unstable-msc2676")]
use super::RoomMessageEventContent;
#[cfg(feature = "unstable-msc3440")]
use super::Thread;
use super::{InReplyTo, Relation};
#[cfg(any(feature = "unstable-msc2676", feature = "unstable-msc3440"))]
use crate::OwnedEventId;

impl<'de> Deserialize<'de> for Relation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ev = EventWithRelatesToJsonRepr::deserialize(deserializer)?;

        #[cfg(feature = "unstable-msc3440")]
        if let Some(
            RelationJsonRepr::ThreadStable(ThreadStableJsonRepr { event_id, is_falling_back })
            | RelationJsonRepr::ThreadUnstable(ThreadUnstableJsonRepr { event_id, is_falling_back }),
        ) = ev.relates_to.relation
        {
            let in_reply_to = ev
                .relates_to
                .in_reply_to
                .ok_or_else(|| serde::de::Error::missing_field("m.in_reply_to"))?;
            return Ok(Relation::Thread(Thread { event_id, in_reply_to, is_falling_back }));
        }

        let rel = if let Some(in_reply_to) = ev.relates_to.in_reply_to {
            Relation::Reply { in_reply_to }
        } else if let Some(relation) = ev.relates_to.relation {
            match relation {
                #[cfg(feature = "unstable-msc2676")]
                RelationJsonRepr::Replacement(ReplacementJsonRepr { event_id }) => {
                    let new_content = ev
                        .new_content
                        .ok_or_else(|| serde::de::Error::missing_field("m.new_content"))?;
                    Relation::Replacement(Replacement { event_id, new_content })
                }
                // FIXME: Maybe we should log this, though at this point we don't even have
                // access to the rel_type of the unknown relation.
                RelationJsonRepr::Unknown => Relation::_Custom,
                #[cfg(feature = "unstable-msc3440")]
                RelationJsonRepr::ThreadStable(_) | RelationJsonRepr::ThreadUnstable(_) => {
                    unreachable!()
                }
            }
        } else {
            Relation::_Custom
        };

        Ok(rel)
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
            #[cfg(feature = "unstable-msc3440")]
            Relation::Thread(Thread { event_id, in_reply_to, is_falling_back }) => {
                EventWithRelatesToJsonRepr::new(RelatesToJsonRepr {
                    in_reply_to: Some(in_reply_to.clone()),
                    relation: Some(RelationJsonRepr::ThreadUnstable(ThreadUnstableJsonRepr {
                        event_id: event_id.clone(),
                        is_falling_back: *is_falling_back,
                    })),
                    ..Default::default()
                })
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

/// Struct modeling the different ways relationships can be expressed in a `m.relates_to` field of
/// an event.
#[derive(Default, Deserialize, Serialize)]
struct RelatesToJsonRepr {
    #[serde(rename = "m.in_reply_to", skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<InReplyTo>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    relation: Option<RelationJsonRepr>,
}

impl RelatesToJsonRepr {
    fn is_empty(&self) -> bool {
        self.in_reply_to.is_none() && self.relation.is_none()
    }
}

/// A relation, which associates new information to an existing event.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "rel_type")]
enum RelationJsonRepr {
    /// An event that replaces another event.
    #[cfg(feature = "unstable-msc2676")]
    #[serde(rename = "m.replace")]
    Replacement(ReplacementJsonRepr),

    /// An event that belongs to a thread, with unstable names.
    #[cfg(feature = "unstable-msc3440")]
    #[serde(rename = "m.thread")]
    ThreadStable(ThreadStableJsonRepr),

    /// An event that belongs to a thread, with unstable names.
    #[cfg(feature = "unstable-msc3440")]
    #[serde(rename = "io.element.thread")]
    ThreadUnstable(ThreadUnstableJsonRepr),

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
    event_id: OwnedEventId,
}

/// A thread relation without the reply fallback, with stable names.
#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc3440")]
struct ThreadStableJsonRepr {
    /// The ID of the root message in the thread.
    event_id: OwnedEventId,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a real reply in a
    /// thread.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    is_falling_back: bool,
}

/// A thread relation without the reply fallback, with unstable names.
#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc3440")]
struct ThreadUnstableJsonRepr {
    /// The ID of the root message in the thread.
    event_id: OwnedEventId,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a real reply in a
    /// thread.
    #[serde(
        rename = "io.element.show_reply",
        default,
        skip_serializing_if = "ruma_common::serde::is_default"
    )]
    is_falling_back: bool,
}
