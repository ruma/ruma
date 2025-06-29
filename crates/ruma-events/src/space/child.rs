//! Types for the [`m.space.child`] event.
//!
//! [`m.space.child`]: https://spec.matrix.org/latest/client-server-api/#mspacechild

use std::{cmp::Ordering, ops::Deref};

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedRoomId, OwnedServerName, OwnedUserId, RoomId};
use ruma_macros::{Event, EventContent};
use serde::{de, Deserialize, Serialize};

use crate::{StateEvent, SyncStateEvent};

/// The content of an `m.space.child` event.
///
/// The admins of a space can advertise rooms and subspaces for their space by setting
/// `m.space.child` state events.
///
/// The `state_key` is the ID of a child room or space, and the content must contain a `via` key
/// which gives a list of candidate servers that can be used to join the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.space.child", kind = State, state_key_type = OwnedRoomId)]
pub struct SpaceChildEventContent {
    /// List of candidate servers that can be used to join the room.
    pub via: Vec<OwnedServerName>,

    /// Provide a default ordering of siblings in the room list.
    ///
    /// Rooms are sorted based on a lexicographic ordering of the Unicode codepoints of the
    /// characters in `order` values. Rooms with no `order` come last, in ascending numeric order
    /// of the origin_server_ts of their m.room.create events, or ascending lexicographic order of
    /// their room_ids in case of equal `origin_server_ts`. `order`s which are not strings, or do
    /// not consist solely of ascii characters in the range `\x20` (space) to `\x7E` (`~`), or
    /// consist of more than 50 characters, are forbidden and the field should be ignored if
    /// received.
    ///
    /// During deserialization, this field is set to `None` if it is empty or invalid.
    #[serde(
        default,
        deserialize_with = "deserialize_order",
        skip_serializing_if = "Option::is_none"
    )]
    pub order: Option<String>,

    /// Space admins can mark particular children of a space as "suggested".
    ///
    /// This mainly serves as a hint to clients that that they can be displayed differently, for
    /// example by showing them eagerly in the room list. A child which is missing the `suggested`
    /// property is treated identically to a child with `"suggested": false`. A suggested child may
    /// be a room or a subspace.
    ///
    /// Defaults to `false`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub suggested: bool,
}

impl SpaceChildEventContent {
    /// Creates a new `SpaceChildEventContent` with the given routing servers.
    pub fn new(via: Vec<OwnedServerName>) -> Self {
        Self { via, order: None, suggested: false }
    }

    /// Check whether the given `order` is valid according to the Matrix specification.
    ///
    /// According to the specification, the order:
    ///
    /// > Must consist of ASCII characters within the range `\x20` (space) and `\x7E` (~),
    /// > inclusive. Must not exceed 50 characters.
    ///
    /// Returns `Ok(())` if the order passes validation, or an error if the order doesn't respect
    /// the rules from the spec or is empty, as it cannot be used for ordering.
    pub fn validate_order(order: &str) -> Result<(), OrderValidationError> {
        if order.is_empty() {
            return Err(OrderValidationError::Empty);
        }

        if order.len() > 50 {
            return Err(OrderValidationError::MaximumLengthExceeded);
        }

        if !order.bytes().all(|byte| (b'\x20'..=b'\x7E').contains(&byte)) {
            return Err(OrderValidationError::InvalidCharacters);
        }

        Ok(())
    }
}

/// Deserialize the `order` of [`SpaceChildEventContent`].
///
/// According to the spec, order is a string that:
///
/// > Must consist of ASCII characters within the range \x20 (space) and \x7E (~), inclusive. Must
/// > not exceed 50 characters.
///
/// This function never returns an error, `Ok(None)` is returned if validation of the order fails.
fn deserialize_order<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let Ok(Some(order)) = Option::<String>::deserialize(deserializer) else {
        return Ok(None);
    };

    if SpaceChildEventContent::validate_order(&order).is_err() {
        return Ok(None);
    }

    Ok(Some(order))
}

/// An error encountered when trying to validate the `order` of a [`SpaceChildEventContent`].
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum OrderValidationError {
    /// The order is empty.
    #[error("order is empty")]
    Empty,

    /// The order contains invalid characters.
    #[error("order contains invalid characters")]
    InvalidCharacters,

    /// The order exceeds 50 bytes.
    #[error("order exceeds 50 bytes")]
    MaximumLengthExceeded,
}

/// An `m.space.child` event represented as a Stripped State Event with an added `origin_server_ts`
/// key.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct HierarchySpaceChildEvent {
    /// The content of the space child event.
    pub content: SpaceChildEventContent,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// The room ID of the child.
    pub state_key: OwnedRoomId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
}

/// Helper trait to sort `m.space.child` events using using the algorithm for [ordering children
/// within a space].
///
/// This trait can be used to sort a slice using `.sort_by(SpaceChildOrd::cmp_space_child)`. It is
/// also possible to use [`SpaceChildOrdHelper`] to sort the events in a `BTreeMap` or a `BTreeSet`.
///
/// [ordering children within a space]: https://spec.matrix.org/latest/client-server-api/#ordering-of-children-within-a-space
pub trait SpaceChildOrd {
    #[doc(hidden)]
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_>;

    /// Return an [`Ordering`] between `self` and `other`, using the algorithm for [ordering
    /// children within a space].
    ///
    /// [ordering children within a space]: https://spec.matrix.org/latest/client-server-api/#ordering-of-children-within-a-space
    fn cmp_space_child(&self, other: &impl SpaceChildOrd) -> Ordering {
        self.space_child_ord_fields().cmp(&other.space_child_ord_fields())
    }
}

/// Fields necessary to implement `Ord` for space child events using the algorithm for [ordering
/// children within a space].
///
/// [ordering children within a space]: https://spec.matrix.org/latest/client-server-api/#ordering-of-children-within-a-space
#[doc(hidden)]
#[derive(PartialEq, Eq)]
pub struct SpaceChildOrdFields<'a> {
    order: Option<&'a str>,
    origin_server_ts: MilliSecondsSinceUnixEpoch,
    state_key: &'a RoomId,
}

impl<'a> SpaceChildOrdFields<'a> {
    /// Construct a new `SpaceChildEventOrdFields` with the given values.
    ///
    /// Filters the order if it is invalid.
    fn new(
        order: Option<&'a str>,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
        state_key: &'a RoomId,
    ) -> Self {
        Self {
            order: order.filter(|order| SpaceChildEventContent::validate_order(order).is_ok()),
            origin_server_ts,
            state_key,
        }
    }
}

impl<'a> Ord for SpaceChildOrdFields<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.order, other.order) {
            // Events with order are ordered before events without order.
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(self_order), Some(other_order)) => self_order.cmp(other_order),
            (None, None) => Ordering::Equal,
        }
        .then(self.origin_server_ts.cmp(&other.origin_server_ts))
        .then(self.state_key.cmp(other.state_key))
    }
}

impl<'a> PartialOrd for SpaceChildOrdFields<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> SpaceChildOrd for &T
where
    T: SpaceChildOrd,
{
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        (*self).space_child_ord_fields()
    }
}

impl SpaceChildOrd for OriginalSpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        SpaceChildOrdFields::new(
            self.content.order.as_deref(),
            self.origin_server_ts,
            &self.state_key,
        )
    }
}

impl SpaceChildOrd for RedactedSpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        SpaceChildOrdFields::new(None, self.origin_server_ts, &self.state_key)
    }
}

impl SpaceChildOrd for SpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        match self {
            StateEvent::Original(original) => original.space_child_ord_fields(),
            StateEvent::Redacted(redacted) => redacted.space_child_ord_fields(),
        }
    }
}

impl SpaceChildOrd for OriginalSyncSpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        SpaceChildOrdFields::new(
            self.content.order.as_deref(),
            self.origin_server_ts,
            &self.state_key,
        )
    }
}

impl SpaceChildOrd for RedactedSyncSpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        SpaceChildOrdFields::new(None, self.origin_server_ts, &self.state_key)
    }
}

impl SpaceChildOrd for SyncSpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        match self {
            SyncStateEvent::Original(original) => original.space_child_ord_fields(),
            SyncStateEvent::Redacted(redacted) => redacted.space_child_ord_fields(),
        }
    }
}

impl SpaceChildOrd for HierarchySpaceChildEvent {
    fn space_child_ord_fields(&self) -> SpaceChildOrdFields<'_> {
        SpaceChildOrdFields::new(
            self.content.order.as_deref(),
            self.origin_server_ts,
            &self.state_key,
        )
    }
}

/// Helper trait to sort `m.space.child` events using using the algorithm for [ordering children
/// within a space].
///
/// This type can be use with `BTreeMap` or `BTreeSet` to order space child events.
///
/// [ordering children within a space]: https://spec.matrix.org/latest/client-server-api/#ordering-of-children-within-a-space
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct SpaceChildOrdHelper<T: SpaceChildOrd>(pub T);

impl<T: SpaceChildOrd> PartialEq for SpaceChildOrdHelper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.space_child_ord_fields().eq(&other.0.space_child_ord_fields())
    }
}

impl<T: SpaceChildOrd> Eq for SpaceChildOrdHelper<T> {}

impl<T: SpaceChildOrd> Ord for SpaceChildOrdHelper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp_space_child(&other.0)
    }
}

impl<T: SpaceChildOrd> PartialOrd for SpaceChildOrdHelper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: SpaceChildOrd> Deref for SpaceChildOrdHelper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, iter::repeat_n};

    use js_int::{uint, UInt};
    use ruma_common::{
        owned_server_name, owned_user_id, room_id, server_name, MilliSecondsSinceUnixEpoch, RoomId,
    };
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        HierarchySpaceChildEvent, OrderValidationError, SpaceChildEventContent, SpaceChildOrd,
        SpaceChildOrdHelper,
    };

    #[test]
    fn space_child_serialization() {
        let content = SpaceChildEventContent {
            via: vec![server_name!("example.com").to_owned()],
            order: Some("uwu".to_owned()),
            suggested: false,
        };

        let json = json!({
            "via": ["example.com"],
            "order": "uwu",
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn space_child_empty_serialization() {
        let content = SpaceChildEventContent { via: vec![], order: None, suggested: false };

        let json = json!({ "via": [] });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn space_child_content_deserialization_order() {
        let via = server_name!("localhost");

        // Valid string.
        let json = json!({
            "order": "aaa",
            "via": [via],
        });
        let content = from_json_value::<SpaceChildEventContent>(json).unwrap();
        assert_eq!(content.order.as_deref(), Some("aaa"));
        assert!(!content.suggested);
        assert_eq!(content.via, &[via]);

        // Not a string.
        let json = json!({
            "order": 2,
            "via": [via],
        });
        let content = from_json_value::<SpaceChildEventContent>(json).unwrap();
        assert_eq!(content.order, None);
        assert!(!content.suggested);
        assert_eq!(content.via, &[via]);

        // Empty string.
        let json = json!({
            "order": "",
            "via": [via],
        });
        let content = from_json_value::<SpaceChildEventContent>(json).unwrap();
        assert_eq!(content.order, None);
        assert!(!content.suggested);
        assert_eq!(content.via, &[via]);

        // String too long.
        let order = repeat_n('a', 60).collect::<String>();
        let json = json!({
            "order": order,
            "via": [via],
        });
        let content = from_json_value::<SpaceChildEventContent>(json).unwrap();
        assert_eq!(content.order, None);
        assert!(!content.suggested);
        assert_eq!(content.via, &[via]);

        // Invalid character.
        let json = json!({
            "order": "🔝",
            "via": [via],
        });
        let content = from_json_value::<SpaceChildEventContent>(json).unwrap();
        assert_eq!(content.order, None);
        assert!(!content.suggested);
        assert_eq!(content.via, &[via]);
    }

    #[test]
    fn hierarchy_space_child_deserialization() {
        let json = json!({
            "content": {
                "via": [
                    "example.org"
                ]
            },
            "origin_server_ts": 1_629_413_349,
            "sender": "@alice:example.org",
            "state_key": "!a:example.org",
            "type": "m.space.child"
        });

        let ev = from_json_value::<HierarchySpaceChildEvent>(json).unwrap();
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1_629_413_349)));
        assert_eq!(ev.sender, "@alice:example.org");
        assert_eq!(ev.state_key, "!a:example.org");
        assert_eq!(ev.content.via, ["example.org"]);
        assert_eq!(ev.content.order, None);
        assert!(!ev.content.suggested);
    }

    #[test]
    fn validate_space_child_order() {
        // Valid string.
        SpaceChildEventContent::validate_order("aaa").unwrap();

        // Empty string.
        assert_eq!(SpaceChildEventContent::validate_order(""), Err(OrderValidationError::Empty));

        // String too long.
        let order = repeat_n('a', 60).collect::<String>();
        assert_eq!(
            SpaceChildEventContent::validate_order(&order),
            Err(OrderValidationError::MaximumLengthExceeded)
        );

        // Invalid character.
        assert_eq!(
            SpaceChildEventContent::validate_order("🔝"),
            Err(OrderValidationError::InvalidCharacters)
        );
    }

    /// Construct a [`HierarchySpaceChildEvent`] with the given state key, order and timestamp.
    fn hierarchy_space_child_event(
        state_key: &RoomId,
        order: Option<&str>,
        origin_server_ts: UInt,
    ) -> HierarchySpaceChildEvent {
        let mut content = SpaceChildEventContent::new(vec![owned_server_name!("example.org")]);
        content.order = order.map(ToOwned::to_owned);

        HierarchySpaceChildEvent {
            content,
            sender: owned_user_id!("@alice:example.org"),
            state_key: state_key.to_owned(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(origin_server_ts),
        }
    }

    #[test]
    fn space_child_ord_spec_example() {
        // Reproduce the example from the spec.
        let child_a = hierarchy_space_child_event(
            room_id!("!a:example.org"),
            Some("aaaa"),
            uint!(1_640_141_000),
        );
        let child_b = hierarchy_space_child_event(
            room_id!("!b:example.org"),
            Some(" "),
            uint!(1_640_341_000),
        );
        let child_c = hierarchy_space_child_event(
            room_id!("!c:example.org"),
            Some("first"),
            uint!(1_640_841_000),
        );
        let child_d =
            hierarchy_space_child_event(room_id!("!d:example.org"), None, uint!(1_640_741_000));
        let child_e =
            hierarchy_space_child_event(room_id!("!e:example.org"), None, uint!(1_640_641_000));

        let events =
            [child_a.clone(), child_b.clone(), child_c.clone(), child_d.clone(), child_e.clone()];

        // Using slice::sort_by.
        let mut sorted_events = events.clone();
        sorted_events.sort_by(SpaceChildOrd::cmp_space_child);
        assert_eq!(sorted_events[0].state_key, child_b.state_key);
        assert_eq!(sorted_events[1].state_key, child_a.state_key);
        assert_eq!(sorted_events[2].state_key, child_c.state_key);
        assert_eq!(sorted_events[3].state_key, child_e.state_key);
        assert_eq!(sorted_events[4].state_key, child_d.state_key);

        // Using BTreeSet.
        let sorted_events = events.into_iter().map(SpaceChildOrdHelper).collect::<BTreeSet<_>>();
        let mut iter = sorted_events.iter();
        assert_eq!(iter.next().unwrap().state_key, child_b.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_a.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_c.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_e.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_d.state_key);
    }

    #[test]
    fn space_child_ord_other_example() {
        // We also check invalid order and state key comparison here.
        let child_a = hierarchy_space_child_event(
            room_id!("!a:example.org"),
            Some("🔝"),
            uint!(1_640_141_000),
        );
        let child_b = hierarchy_space_child_event(
            room_id!("!b:example.org"),
            Some(" "),
            uint!(1_640_341_000),
        );
        let child_c =
            hierarchy_space_child_event(room_id!("!c:example.org"), Some(""), uint!(1_640_841_000));
        let child_d =
            hierarchy_space_child_event(room_id!("!d:example.org"), None, uint!(1_640_741_000));
        let child_e =
            hierarchy_space_child_event(room_id!("!e:example.org"), None, uint!(1_640_741_000));

        let mut events =
            [child_a.clone(), child_b.clone(), child_c.clone(), child_d.clone(), child_e.clone()];

        events.sort_by(SpaceChildOrd::cmp_space_child);

        // Using slice::sort_by.
        let mut sorted_events = events.clone();
        sorted_events.sort_by(SpaceChildOrd::cmp_space_child);
        assert_eq!(sorted_events[0].state_key, child_b.state_key);
        assert_eq!(sorted_events[1].state_key, child_a.state_key);
        assert_eq!(sorted_events[2].state_key, child_d.state_key);
        assert_eq!(sorted_events[3].state_key, child_e.state_key);
        assert_eq!(sorted_events[4].state_key, child_c.state_key);

        // Using BTreeSet.
        let sorted_events = events.into_iter().map(SpaceChildOrdHelper).collect::<BTreeSet<_>>();
        let mut iter = sorted_events.iter();
        assert_eq!(iter.next().unwrap().state_key, child_b.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_a.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_d.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_e.state_key);
        assert_eq!(iter.next().unwrap().state_key, child_c.state_key);
    }
}
