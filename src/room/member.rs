//! Types for the *m.room.member* event.

use std::collections::HashMap;

use core::{Event, EventType, RoomEvent, StateEvent, StrippedState, StrippedStateType};

/// The current membership state of a user in the room.
///
/// Adjusts the membership state for a user in a room. It is preferable to use the membership APIs
/// (``/rooms/<room id>/invite`` etc) when performing membership actions rather than adjusting the
/// state directly as there are a restricted set of valid transformations. For example, user A
/// cannot force user B to join a room, and trying to force this state change directly will fail.
///
/// The *third_party_invite* property will be set if this invite is an *invite* event and is the
/// successor of an *m.room.third_party_invite* event, and absent otherwise.
///
/// This event may also include an *invite_room_state* key outside the *content* key. If present,
/// this contains an array of `StrippedState` events. These events provide information on a few
/// select state events such as the room name.
pub struct MemberEvent<'a, 'b, T: 'a> {
    content: MemberEventContent<'a>,
    event_id: &'a str,
    invite_room_state: Option<&'a[&'a StrippedState<'a, T>]>,
    prev_content: Option<MemberEventContent<'b>>,
    room_id: &'a str,
    state_key: &'a str,
    user_id: &'a str,
}

impl<'a, 'b, T> Event<'a, MemberEventContent<'a>> for MemberEvent<'a, 'b, T> {
    fn content(&'a self) -> &'a MemberEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomMember
    }
}

impl<'a, 'b, T> RoomEvent<'a, MemberEventContent<'a>> for MemberEvent<'a, 'b, T> {
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

impl<'a, 'b, T> StateEvent<'a, 'b, MemberEventContent<'a>> for MemberEvent<'a, 'b, T> {
    fn prev_content(&'a self) -> Option<&'b MemberEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }

    fn state_key(&self) -> &'a str {
        &self.state_key
    }
}

/// The payload of a `MemberEvent`.
pub struct MemberEventContent<'a> {
    avatar_url: Option<&'a str>,
    displayname: Option<&'a str>,
    membership: MembershipState,
    third_party_invite: (), // TODO
}

/// The membership state of a user.
pub enum MembershipState {
    Ban,
    Invite,
    Join,
    Knock,
    Leave,
}
