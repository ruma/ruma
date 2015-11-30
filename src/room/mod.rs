//! Events within the *m.room* namespace.

pub mod message;

use core::{Event, RoomEvent, StateEvent};

/// Informs the room about what room aliases it has been given.
pub struct AliasesEvent<'a, 'b> {
    content: AliasesEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<AliasesEventContent<'b>>,
    room_id: &'a str,
    /// The homeserver domain which owns these room aliases.
    state_key: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, AliasesEventContent<'a>> for AliasesEvent<'a, 'b> {
    fn content(&'a self) -> &'a AliasesEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.aliases"
    }
}

impl<'a, 'b> RoomEvent<'a, AliasesEventContent<'a>> for AliasesEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, AliasesEventContent<'a>> for AliasesEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b AliasesEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }

    fn state_key(&self) -> &'a str {
        &self.state_key
    }
}

/// The payload of an `AliasesEvent`.
pub struct AliasesEventContent<'a> {
    /// A list of room aliases.
    aliases: &'a[&'a str],
}

/// Informs the room as to which alias is the canonical one.
pub struct CanonicalAliasEvent<'a, 'b> {
    content: CanonicalAliasEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<CanonicalAliasEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, CanonicalAliasEventContent<'a>> for CanonicalAliasEvent<'a, 'b> {
    fn content(&'a self) -> &'a CanonicalAliasEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.canonical_alias"
    }
}

impl<'a, 'b> RoomEvent<'a, CanonicalAliasEventContent<'a>> for CanonicalAliasEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, CanonicalAliasEventContent<'a>> for CanonicalAliasEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b CanonicalAliasEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `CanonicalAliasEvent`.
pub struct CanonicalAliasEventContent<'a> {
    /// The canonical alias.
    alias: &'a str,
}

/// This is the first event in a room and cannot be changed. It acts as the root of all other
/// events.
pub struct CreateEvent<'a, 'b> {
    content: CreateEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<CreateEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, CreateEventContent<'a>> for CreateEvent<'a, 'b> {
    fn content(&'a self) -> &'a CreateEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.create"
    }
}

impl<'a, 'b> RoomEvent<'a, CreateEventContent<'a>> for CreateEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, CreateEventContent<'a>> for CreateEvent<'a, 'b> {}

/// The payload of a `CreateEvent`.
pub struct CreateEventContent<'a> {
    /// The `user_id` of the room creator. This is set by the homeserver.
    creator: &'a str,
}

/// This event controls whether a member of a room can see the events that happened in a room from
/// before they joined.
pub struct HistoryVisibilityEvent<'a, 'b> {
    content: HistoryVisibilityEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<HistoryVisibilityEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, HistoryVisibilityEventContent<'a>> for HistoryVisibilityEvent<'a, 'b> {
    fn content(&'a self) -> &'a HistoryVisibilityEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.history_visibility"
    }
}

impl<'a, 'b> RoomEvent<'a, HistoryVisibilityEventContent<'a>> for HistoryVisibilityEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, HistoryVisibilityEventContent<'a>>
    for HistoryVisibilityEvent<'a, 'b>
{
    fn prev_content(&'a self) -> Option<&'b HistoryVisibilityEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `HistoryVisibilityEvent`.
pub struct HistoryVisibilityEventContent<'a> {
    /// Who can see the room history.
    history_visibility: &'a HistoryVisibility,
}

/// Who can see a room's history.
pub enum HistoryVisibility {
    /// Previous events are accessible to newly joined members from the point they were invited
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *invite* or *join*.
    Invited,
    /// Previous events are accessible to newly joined members from the point they joined the room
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *join*.
    Joined,
    /// Previous events are always accessible to newly joined members. All events in the room are
    /// accessible, even those sent when the member was not a part of the room.
    Shared,
    /// All events while this is the `HistoryVisibility` value may be shared by any
    /// participating homeserver with anyone, regardless of whether they have ever joined the room.
    WorldReadable,
}

/// Describes how users are allowed to join the room.
pub struct JoinRulesEvent<'a, 'b> {
    content: JoinRulesEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<JoinRulesEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, JoinRulesEventContent<'a>> for JoinRulesEvent<'a, 'b> {
    fn content(&'a self) -> &'a JoinRulesEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.join_rules"
    }
}

impl<'a, 'b> RoomEvent<'a, JoinRulesEventContent<'a>> for JoinRulesEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, JoinRulesEventContent<'a>> for JoinRulesEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b JoinRulesEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `JoinRulesEvent`.
pub struct JoinRulesEventContent<'a> {
    /// The type of rules used for users wishing to join this room.
    join_rule: &'a JoinRules,
}

/// The rule used for users wishing to join this room.
pub enum JoinRules {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,
    /// Reserved but not yet implemented by the Matrix specification.
    Knock,
    /// Reserved but not yet implemented by the Matrix specification.
    Private,
    /// Anyone can join the room without any prior action.
    Public,
}
