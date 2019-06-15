//! Types for the *m.room.member* event.

use ruma_identifiers::UserId;
use ruma_signatures::Signatures;
use serde::{Deserialize, Serialize};

state_event! {
    /// The current membership state of a user in the room.
    ///
    /// Adjusts the membership state for a user in a room. It is preferable to use the membership
    /// APIs (`/rooms/<room id>/invite` etc) when performing membership actions rather than
    /// adjusting the state directly as there are a restricted set of valid transformations. For
    /// example, user A cannot force user B to join a room, and trying to force this state change
    /// directly will fail.
    ///
    /// The `third_party_invite` property will be set if this invite is an *invite* event and is the
    /// successor of an *m.room.third_party_invite* event, and absent otherwise.
    ///
    /// This event may also include an `invite_room_state` key inside the event's unsigned data. If
    /// present, this contains an array of `StrippedState` events. These events provide information
    /// on a subset of state events such as the room name. Note that ruma-events treats unsigned
    /// data on events as arbitrary JSON values, and the ruma-events types for this event don't
    /// provide direct access to these `invite_room_state`. If you need this data, you must extract
    /// and convert it from a `serde_json::Value` yourself.
    ///
    /// The user for which a membership applies is represented by the `state_key`. Under some
    /// conditions, the `sender` and `state_key` may not match - this may be interpreted as the
    /// `sender` affecting the membership state of the `state_key` user.
    ///
    /// The membership for a given user can change over time. Previous membership can be retrieved
    /// from the `prev_content` object on an event. If not present, the user's previous membership
    /// must be assumed as leave.
    pub struct MemberEvent(MemberEventContent) {}
}

/// The payload of a `MemberEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemberEventContent {
    /// The avatar URL for this user, if any. This is added by the homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// The display name for this user, if any. This is added by the homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayname: Option<String>,

    /// Flag indicating if the room containing this event was created
    /// with the intention of being a direct chat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_direct: Option<bool>,

    /// The membership state of this user.
    pub membership: MembershipState,

    /// If this member event is the successor to a third party invitation, this field will contain
    /// information about that invitation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_invite: Option<ThirdPartyInvite>,
}

/// The membership state of a user.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum MembershipState {
    /// The user is banned.
    #[serde(rename = "ban")]
    Ban,

    /// The user has been invited.
    #[serde(rename = "invite")]
    Invite,

    /// The user has joined.
    #[serde(rename = "join")]
    Join,

    /// The user has requested to join.
    #[serde(rename = "knock")]
    Knock,

    /// The user has left.
    #[serde(rename = "leave")]
    Leave,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
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

/// Information about a third party invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThirdPartyInvite {
    /// A name which can be displayed to represent the user instead of their third party
    /// identifier.
    pub display_name: String,

    /// A block of content which has been signed, which servers can use to verify the event.
    /// Clients should ignore this.
    pub signed: SignedContent,
}

/// A block of content which has been signed, which servers can use to verify a third party
/// invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedContent {
    /// The invited Matrix user ID.
    ///
    /// Must be equal to the user_id property of the event.
    pub mxid: UserId,

    /// A single signature from the verifying server, in the format specified by the Signing Events
    /// section of the server-server API.
    pub signatures: Signatures,

    /// The token property of the containing third_party_invite object.
    pub token: String,
}
