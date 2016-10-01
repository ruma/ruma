//! Types for the *m.room.member* event.

use StateEvent;
use stripped::StrippedState;

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
pub type MemberEvent = StateEvent<MemberEventContent, MemberEventExtraContent>;

/// The payload of a `MemberEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct MemberEventContent {
    /// The avatar URL for this user.
    #[serde(skip_serializing_if="Option::is_none")]
    pub avatar_url: Option<String>,

    /// The display name for this user.
    #[serde(skip_serializing_if="Option::is_none")]
    pub displayname: Option<String>,

    /// The membership state of this user.
    pub membership: MembershipState,

    /// Warning: This field is not implemented yet and its type will change!
    pub third_party_invite: (), // TODO
}

/// The membership state of a user.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum MembershipState {
    /// The user is banned.
    #[serde(rename="ban")]
    Ban,

    /// The user has been invited.
    #[serde(rename="invite")]
    Invite,

    /// The user has joined.
    #[serde(rename="join")]
    Join,

    /// The user has requested to join.
    #[serde(rename="knock")]
    Knock,

    /// The user has left.
    #[serde(rename="leave")]
    Leave,
}

/// Extra content for a `MemberEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct MemberEventExtraContent {
    /// A subset of the state of the room at the time of the invite.
    #[serde(skip_serializing_if="Option::is_none")]
    pub invite_room_state: Option<Vec<StrippedState>>,
}

impl_enum! {
    MembershipState {
        Ban => "ban",
        Invite => "invite",
        Join => "join",
        Knock => "knock",
        Leave => "leave",
    }
}
