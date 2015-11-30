//! Types for the basic kinds of events.

/// The type of an event.
pub enum EventType {
    CallAnswer,
    CallCandidates,
    CallHangup,
    CallInvite,
    Presence,
    Receipt,
    RoomAliases,
    RoomAvatar,
    RoomCanonicalAlias,
    RoomCreate,
    RoomGuestAccess,
    RoomHistoryVisibility,
    RoomJoinRules,
    RoomMember,
    RoomMessage,
    RoomMessageFeedback,
    RoomName,
    RoomPowerLevels,
    RoomRedaction,
    RoomThirdPartyInvite,
    RoomTopic,
    Tag,
    Typing,
}

/// Functionality common to all events.
pub trait Event<'a, T> {
    /// The primary event payload.
    fn content(&'a self) -> &'a T;
    /// The type of event.
    fn event_type(&self) -> EventType;
}

/// An event emitted within the context of a room.
pub trait RoomEvent<'a, T>: Event<'a, T> {
    /// The globally unique event identifier.
    fn event_id(&'a self) -> &'a str;
    /// The ID of the room associated with this event.
    fn room_id(&'a self) -> &'a str;
    /// The fully-qualified ID of the user who sent the event.
    fn user_id(&'a self) -> &'a str;
}

/// An event that represents some aspect of a room's state.
pub trait StateEvent<'a, 'b, T>: RoomEvent<'a, T> {
    /// Previous content for this aspect of room state.
    fn prev_content(&'a self) -> Option<&'b T> {
        None
    }

    /// A unique key which defines the overwriting semantics for this aspect of room state.
    fn state_key(&self) -> &'a str {
        ""
    }
}
