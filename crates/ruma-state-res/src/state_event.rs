use std::sync::Arc;

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, UserId};

/// Abstraction of a PDU so users can have their own PDU types.
pub trait Event {
    /// The `EventId` of this event.
    fn event_id(&self) -> &EventId;

    /// The `RoomId` of this event.
    fn room_id(&self) -> &RoomId;

    /// The `UserId` of this event.
    fn sender(&self) -> &UserId;

    /// The time of creation on the originating server.
    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch;

    /// The event type.
    fn event_type(&self) -> &EventType;

    /// The event's content.
    // FIXME: This forces a serde_json::Value to be stored, which the previous solution of returning
    // an owned one did not. However, the previous signature was even less efficient and also
    // heavily encouraged storing `serde_json::Value`. We should likely force usage of `RawValue`
    // instead, or somehow allow different storage without pessimizing all but one.
    fn content(&self) -> &serde_json::Value;

    /// The state key for this event.
    fn state_key(&self) -> Option<&str>;

    /// The events before this event.
    // Requires GATs to avoid boxing (and TAIT for making it convenient).
    fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_>;

    /// All the authenticating events for this event.
    // Requires GATs to avoid boxing (and TAIT for making it convenient).
    fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_>;

    /// If this event is a redaction event this is the event it redacts.
    fn redacts(&self) -> Option<&EventId>;
}

impl<T: Event> Event for &T {
    fn event_id(&self) -> &EventId {
        (*self).event_id()
    }

    fn room_id(&self) -> &RoomId {
        (*self).room_id()
    }

    fn sender(&self) -> &UserId {
        (*self).sender()
    }

    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        (*self).origin_server_ts()
    }

    fn event_type(&self) -> &EventType {
        (*self).event_type()
    }

    fn content(&self) -> &serde_json::Value {
        (*self).content()
    }

    fn state_key(&self) -> Option<&str> {
        (*self).state_key()
    }

    fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_> {
        (*self).prev_events()
    }

    fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_> {
        (*self).auth_events()
    }

    fn redacts(&self) -> Option<&EventId> {
        (*self).redacts()
    }
}

impl<T: Event> Event for Arc<T> {
    fn event_id(&self) -> &EventId {
        (&**self).event_id()
    }

    fn room_id(&self) -> &RoomId {
        (&**self).room_id()
    }

    fn sender(&self) -> &UserId {
        (&**self).sender()
    }

    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        (&**self).origin_server_ts()
    }

    fn event_type(&self) -> &EventType {
        (&**self).event_type()
    }

    fn content(&self) -> &serde_json::Value {
        (&**self).content()
    }

    fn state_key(&self) -> Option<&str> {
        (&**self).state_key()
    }

    fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_> {
        (&**self).prev_events()
    }

    fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_> {
        (&**self).auth_events()
    }

    fn redacts(&self) -> Option<&EventId> {
        (&**self).redacts()
    }
}
