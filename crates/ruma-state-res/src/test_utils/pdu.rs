use std::collections::BTreeSet;

use js_int::uint;
use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId, UserId,
};
use ruma_events::TimelineEventType;
use serde::{Deserialize, Serialize};
use serde_json::value::{RawValue as RawJsonValue, to_raw_value as to_raw_json_value};

use crate::Event;

/// A Persistent Data Unit (PDU) with the minimal fields to implement [`Event`].
#[derive(Clone, Debug, Deserialize)]
pub struct Pdu {
    /// The ID of the event.
    pub event_id: OwnedEventId,

    /// The room the event belongs to.
    pub room_id: Option<OwnedRoomId>,

    /// The ID of the user who sent the event.
    pub sender: OwnedUserId,

    /// The timestamp on the originating homeserver when this event was created.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The type of the event.
    #[serde(rename = "type")]
    pub event_type: TimelineEventType,

    /// If this field is set, the event is a state event, and it will replace previous events
    /// with the same `type` and `state_key` in the room state.
    pub state_key: Option<String>,

    /// The content of the event.
    pub content: Box<RawJsonValue>,

    /// Event IDs for the most recent events in the room that the homeserver was aware of when it
    /// made this event.
    pub prev_events: BTreeSet<OwnedEventId>,

    /// Event IDs for the authorization events that would allow this event to be in the room.
    pub auth_events: BTreeSet<OwnedEventId>,

    /// For redaction events, the ID of the event being redacted.
    pub redacts: Option<OwnedEventId>,

    /// Whether this event was rejected for not passing the checks on reception of a PDU.
    pub rejected: bool,
}

impl Pdu {
    /// Construct a `Pdu` with the minimum required fields.
    ///
    /// All the other fields use their default value.
    ///
    /// Panics if the content fails to serialize.
    pub fn with_minimal_fields<T>(
        event_id: OwnedEventId,
        sender: OwnedUserId,
        event_type: TimelineEventType,
        content: T,
    ) -> Self
    where
        T: Serialize,
    {
        let content =
            to_raw_json_value(&content).expect("PDU content should serialize successfully");

        Self {
            event_id,
            room_id: None,
            sender,
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(0)),
            event_type,
            state_key: None,
            content,
            prev_events: BTreeSet::new(),
            auth_events: BTreeSet::new(),
            redacts: None,
            rejected: false,
        }
    }

    /// Construct a `Pdu` with the minimum required fields for a state event.
    ///
    /// All the other fields use their default value.
    ///
    /// Panics if the content fails to serialize.
    pub fn with_minimal_state_fields<T>(
        event_id: OwnedEventId,
        sender: OwnedUserId,
        event_type: TimelineEventType,
        state_key: String,
        content: T,
    ) -> Self
    where
        T: Serialize,
    {
        let mut pdu = Self::with_minimal_fields(event_id, sender, event_type, content);
        pdu.state_key = Some(state_key);
        pdu
    }

    /// Set the content of this PDU by serializing it.
    ///
    /// Panics if the serialization fails.
    pub fn set_content<T>(&mut self, content: T)
    where
        T: Serialize,
    {
        self.content =
            to_raw_json_value(&content).expect("PDU content should serialize successfully");
    }
}

impl Event for Pdu {
    type Id = OwnedEventId;

    fn event_id(&self) -> &Self::Id {
        &self.event_id
    }

    fn room_id(&self) -> Option<&RoomId> {
        self.room_id.as_deref()
    }

    fn sender(&self) -> &UserId {
        &self.sender
    }

    fn event_type(&self) -> &TimelineEventType {
        &self.event_type
    }

    fn content(&self) -> &RawJsonValue {
        &self.content
    }

    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        self.origin_server_ts
    }

    fn state_key(&self) -> Option<&str> {
        self.state_key.as_deref()
    }

    fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
        Box::new(self.prev_events.iter())
    }

    fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
        Box::new(self.auth_events.iter())
    }

    fn redacts(&self) -> Option<&Self::Id> {
        self.redacts.as_ref()
    }

    fn rejected(&self) -> bool {
        self.rejected
    }
}
