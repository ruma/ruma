//! Types for the *m.room.member* event.

use std::collections::HashMap;

use core::{EventType, StrippedState};

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
pub struct MemberEvent {
    content: MemberEventContent,
    event_id: String,
    event_type: EventType,
    invite_room_state: Option<Vec<StrippedState>>,
    prev_content: Option<MemberEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `MemberEvent`.
pub struct MemberEventContent {
    avatar_url: Option<String>,
    displayname: Option<String>,
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
