//! Types for the *m.room.member* event.

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
#[derive(Debug, Deserialize, Serialize)]
pub struct MemberEvent {
    pub content: MemberEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub invite_room_state: Option<Vec<StrippedState>>,
    pub prev_content: Option<MemberEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `MemberEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct MemberEventContent {
    pub avatar_url: Option<String>,
    pub displayname: Option<String>,
    pub membership: MembershipState,
    /// Warning: This field is not implemented yet and its type will change!
    pub third_party_invite: (), // TODO
}

/// The membership state of a user.
#[derive(Debug, Deserialize, Serialize)]
pub enum MembershipState {
    Ban,
    Invite,
    Join,
    Knock,
    Leave,
}
