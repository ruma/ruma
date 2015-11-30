//! Types for the *m.room.join_rules* event.

use core::{Event, RoomEvent, StateEvent};

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
