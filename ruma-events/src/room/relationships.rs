//! Types for event relationships.
//!
//! Events in Matrix can relate to one another in a couple of ways, this module
//! adds types to parse the relationship of an event if any exists.
//!
//! MSC for all the relates_to types except replies:
//! <https://github.com/matrix-org/matrix-doc/pull/2674>

use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Enum modeling the different ways relationships can be expressed in a
/// `m.relates_to` field of an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum RelatesToJsonRepr {
    /// A relation which contains subtypes indicating the type of the
    /// relationship with the `rel_type` field.
    #[cfg(feature = "unstable-pre-spec")]
    Relation(RelationJsonRepr),

    /// An `m.in_reply_to` relationship indicating that the event is a reply to
    /// another event.
    Reply {
        /// Information about another message being replied to.
        #[serde(rename = "m.in_reply_to")]
        in_reply_to: InReplyTo,
    },

    /// Custom, unsupported relationship.
    Custom(JsonValue),
}

/// A relation, which associates new information to an existing event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
#[serde(tag = "rel_type")]
pub(crate) enum RelationJsonRepr {
    /// An annotation to an event.
    #[serde(rename = "m.annotation")]
    Annotation(Annotation),

    /// A reference to another event.
    #[serde(rename = "m.reference")]
    Reference(Reference),

    /// An event that replaces another event.
    #[serde(rename = "m.replace")]
    Replacement(Replacement),
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: EventId,
}

/// A reference to another event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
pub struct Reference {
    /// The event we are referencing.
    pub event_id: EventId,
}

/// An annotation for an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Annotation {
    /// The event that is being annotated.
    pub event_id: EventId,

    /// The annotation.
    pub key: String,
}

/// An event replacing another event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
pub struct Replacement {
    /// The event this event is replacing.
    pub event_id: EventId,
}

#[cfg(test)]
mod test {
    use crate::room::message::Relation;
    use matches::assert_matches;
    use ruma_identifiers::event_id;
    use serde_json::{from_value as from_json_value, json};

    #[test]
    fn reply_deserialize() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "m.in_reply_to": {
                "event_id": event_id,
            }
        });

        assert_matches!(
            from_json_value::<Relation>(json).unwrap(),
            Relation::Reply { in_reply_to }
            if in_reply_to.event_id == event_id
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn reference_deserialize() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "rel_type": "m.reference",
            "event_id": event_id,
        });

        assert_matches!(
            from_json_value::<Relation>(json).unwrap(),
            Relation::Reference(reference)
            if reference.event_id == event_id
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn replacement_deserialization() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "rel_type": "m.replace",
            "event_id": event_id,
        });

        assert_matches!(
            from_json_value::<Relation>(json).unwrap(),
            Relation::Replacement(replacement)
            if replacement.event_id == event_id
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn annotation_deserialize() {
        let event_id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "rel_type": "m.annotation",
            "event_id": event_id,
            "key": "🦛",
        });

        assert_matches!(
            from_json_value::<Relation>(json).unwrap(),
            Relation::Annotation(annotation)
            if annotation.event_id == event_id && annotation.key == "🦛"
        );
    }
}
