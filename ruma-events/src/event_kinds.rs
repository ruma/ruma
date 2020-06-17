use std::time::SystemTime;

use ruma_events_macros::Event;
use ruma_identifiers::{EventId, RoomId, UserId};

use crate::{
    BasicEventContent, EphemeralRoomEventContent, EventContent, MessageEventContent,
    StateEventContent, UnsignedData,
};

/// A basic event – one that consists only of it's type and the `content` object.
#[derive(Clone, Debug, Event)]
pub struct BasicEvent<C: BasicEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

/// Ephemeral room event.
#[derive(Clone, Debug, Event)]
pub struct EphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,
}

/// An ephemeral room event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct EphemeralRoomEventStub<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

/// Message event.
#[derive(Clone, Debug, Event)]
pub struct MessageEvent<C: MessageEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// A message event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct MessageEventStub<C: MessageEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Optional previous content for this event.
    pub prev_content: Option<C>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// A state event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct StateEventStub<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Optional previous content for this event.
    pub prev_content: Option<C>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// A stripped-down state event, used for previews of rooms the user has been
/// invited to.
#[derive(Clone, Debug, Event)]
pub struct StrippedStateEventStub<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,
}

/// An event sent using send-to-device messaging.
#[derive(Clone, Debug, Event)]
pub struct ToDeviceEvent<C: EventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,
}

//
//
//
use ruma_events::{AnyMessageEventContent, AnyStateEventContent};

impl<C: StateEventContent> StateEvent<C> {
    /// Maps a `StateEvent<C>` to  `StateEvent<T>` by applying the given
    /// function to the event.
    pub fn map_event<T, F>(self, func: F) -> StateEvent<T>
    where
        T: StateEventContent,
        F: FnOnce(Self) -> StateEvent<T>,
    {
        func(self)
    }
}

impl StateEvent<AnyStateEventContent> {
    /// Maps a `StateEvent<AnyStateEventContent>` to  `StateEvent<T>` by applying the given
    /// function to the events content.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruma_events::{StateEvent, AnyStateEventContent};
    ///
    /// let general_event = StateEvent {
    ///     content: AnyStateEventContent::RoomName(name_content),
    ///     .. state_event
    /// };
    ///
    /// let specific = general_event.map_content(|c| {
    ///     if let AnyStateEventContent::RoomName(n) = c {
    ///         n
    ///     } else {
    ///         panic!()
    ///     }
    /// });
    /// ```
    pub fn map_content<T, F>(self, func: F) -> StateEvent<T>
    where
        T: StateEventContent,
        F: Fn(AnyStateEventContent) -> T,
    {
        let content = func(self.content);
        let prev_content = if let Some(prev) = self.prev_content { Some(func(prev)) } else { None };
        StateEvent {
            content,
            prev_content,
            room_id: self.room_id,
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            state_key: self.state_key,
            unsigned: self.unsigned,
        }
    }
}

impl<C: MessageEventContent> MessageEvent<C> {
    /// Maps a `MessageEvent<C>` to  `MessageEvent<T>` by applying the given
    /// function to the event.
    pub fn map_event<T, F>(self, func: F) -> MessageEvent<T>
    where
        T: MessageEventContent,
        F: FnOnce(Self) -> MessageEvent<T>,
    {
        func(self)
    }
}

impl MessageEvent<AnyMessageEventContent> {
    /// Maps a `MessageEvent<AnyMessageEventContent>` to  `MessageEvent<T>` by applying the given
    /// function to the events content.
    pub fn map_content<T, F>(self, func: F) -> MessageEvent<T>
    where
        T: MessageEventContent,
        F: Fn(AnyMessageEventContent) -> T,
    {
        let content = func(self.content);
        MessageEvent {
            content,
            room_id: self.room_id,
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            unsigned: self.unsigned,
        }
    }
}

impl MessageEventStub<AnyMessageEventContent> {
    /// Maps a `MessageEventStub<AnyMessageEventContent>` to  `MessageEventStub<T>` by applying the given
    /// function to the events content.
    pub fn map_content<T, F>(self, func: F) -> MessageEventStub<T>
    where
        T: MessageEventContent,
        F: Fn(AnyMessageEventContent) -> T,
    {
        let content = func(self.content);
        MessageEventStub {
            content,
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            unsigned: self.unsigned,
        }
    }

    /// Convert a `MessageEventStub` to a `MessageEvent` by adding a room id.
    pub fn into_full_event<C: MessageEventContent>(
        self,
        room_id: RoomId,
    ) -> MessageEvent<AnyMessageEventContent> {
        let MessageEventStub { content, event_id, sender, origin_server_ts, unsigned } = self;
        MessageEvent { content, room_id, event_id, sender, origin_server_ts, unsigned }
    }
}
