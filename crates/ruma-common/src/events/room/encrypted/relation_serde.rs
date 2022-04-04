use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "unstable-msc2677")]
use super::Annotation;
#[cfg(feature = "unstable-msc2676")]
use super::Replacement;
#[cfg(feature = "unstable-msc3440")]
use super::Thread;
use super::{InReplyTo, Reference, Relation};
#[cfg(feature = "unstable-msc3440")]
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
                #[cfg(feature = "unstable-msc2677")]
                RelationJsonRepr::Annotation(a) => Relation::Annotation(a),
                RelationJsonRepr::Reference(r) => Relation::Reference(r),
                #[cfg(feature = "unstable-msc2676")]
                RelationJsonRepr::Replacement(Replacement { event_id }) => {
                    Relation::Replacement(Replacement { event_id })
                }
                #[cfg(feature = "unstable-msc3440")]
                RelationJsonRepr::ThreadStable(_) | RelationJsonRepr::ThreadUnstable(_) => {
                    unreachable!()
                }
                // FIXME: Maybe we should log this, though at this point we don't even have
                // access to the rel_type of the unknown relation.
                RelationJsonRepr::Unknown => Relation::_Custom,
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
        let relates_to = match self {
            #[cfg(feature = "unstable-msc2677")]
            Relation::Annotation(r) => RelatesToJsonRepr {
                relation: Some(RelationJsonRepr::Annotation(r.clone())),
                ..Default::default()
            },
            Relation::Reference(r) => RelatesToJsonRepr {
                relation: Some(RelationJsonRepr::Reference(r.clone())),
                ..Default::default()
            },
            #[cfg(feature = "unstable-msc2676")]
            Relation::Replacement(r) => RelatesToJsonRepr {
                relation: Some(RelationJsonRepr::Replacement(r.clone())),
                ..Default::default()
            },
            Relation::Reply { in_reply_to } => {
                RelatesToJsonRepr { in_reply_to: Some(in_reply_to.clone()), ..Default::default() }
            }
            #[cfg(feature = "unstable-msc3440")]
            Relation::Thread(Thread { event_id, in_reply_to, is_falling_back }) => {
                RelatesToJsonRepr {
                    in_reply_to: Some(in_reply_to.clone()),
                    relation: Some(RelationJsonRepr::ThreadUnstable(ThreadUnstableJsonRepr {
                        event_id: event_id.clone(),
                        is_falling_back: *is_falling_back,
                    })),
                    ..Default::default()
                }
            }
            Relation::_Custom => RelatesToJsonRepr::default(),
        };

        EventWithRelatesToJsonRepr { relates_to }.serialize(serializer)
    }
}

#[derive(Deserialize, Serialize)]
struct EventWithRelatesToJsonRepr {
    #[serde(rename = "m.relates_to", default, skip_serializing_if = "RelatesToJsonRepr::is_empty")]
    relates_to: RelatesToJsonRepr,
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

/// A thread relation without the reply fallback, with stable names.
#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc3440")]
struct ThreadStableJsonRepr {
    /// The ID of the root message in the thread.
    pub event_id: OwnedEventId,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a real reply in a
    /// thread.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub is_falling_back: bool,
}

/// A thread relation without the reply fallback, with unstable names.
#[derive(Clone, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc3440")]
struct ThreadUnstableJsonRepr {
    /// The ID of the root message in the thread.
    pub event_id: OwnedEventId,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a real reply in a
    /// thread.
    #[serde(
        rename = "io.element.show_reply",
        default,
        skip_serializing_if = "ruma_common::serde::is_default"
    )]
    pub is_falling_back: bool,
}

/// A relation, which associates new information to an existing event.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "rel_type")]
enum RelationJsonRepr {
    /// An annotation to an event.
    #[cfg(feature = "unstable-msc2677")]
    #[serde(rename = "m.annotation")]
    Annotation(Annotation),

    /// A reference to another event.
    #[serde(rename = "m.reference")]
    Reference(Reference),

    /// An event that replaces another event.
    #[cfg(feature = "unstable-msc2676")]
    #[serde(rename = "m.replace")]
    Replacement(Replacement),

    /// An event that belongs to a thread, with stable names.
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
