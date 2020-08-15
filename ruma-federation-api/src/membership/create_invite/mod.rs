//! Endpoint for inviting a remote user to a room

use js_int::UInt;
use ruma_events::{room::member::MemberEventContent, EventType};
use ruma_identifiers::{ServerName, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod v1;
pub mod v2;

/// A simplified event that helps the server identify a room.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrippedState {
    /// The `content` for the event.
    pub content: Value,

    /// The `state_key` for the event.
    pub state_key: String,

    /// The `type` for the event.
    #[serde(rename = "type")]
    pub kind: EventType,

    /// The `sender` for the event.
    pub sender: UserId,
}

/// The invite event sent as a response.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InviteEvent {
    /// The matrix ID of the user who sent the original `m.room.third_party_invite`.
    pub sender: UserId,

    /// The name of the inviting homeserver.
    pub origin: Box<ServerName>,

    /// A timestamp added by the inviting homeserver.
    pub origin_server_ts: UInt,

    /// The event type (should always be `m.room.member`).
    #[serde(rename = "type")]
    pub kind: EventType,

    /// The user ID of the invited member.
    pub state_key: UserId,

    /// The content of the event. Must include a `membership` of invite.
    pub content: MemberEventContent,
}

/// Initial set of fields for `Response`.
pub struct InviteEventInit {
    /// The matrix ID of the user who sent the original `m.room.third_party_invite`.
    pub sender: UserId,

    /// The name of the inviting homeserver.
    pub origin: Box<ServerName>,

    /// A timestamp added by the inviting homeserver.
    pub origin_server_ts: UInt,

    /// The user ID of the invited member.
    pub state_key: UserId,

    /// The content of the event. Must include a `membership` of invite.
    pub content: MemberEventContent,
}

impl From<InviteEventInit> for InviteEvent {
    /// Creates a new `Response` with the given inital values
    fn from(init: InviteEventInit) -> Self {
        InviteEvent {
            sender: init.sender,
            origin: init.origin,
            origin_server_ts: init.origin_server_ts,
            kind: EventType::RoomMember,
            state_key: init.state_key,
            content: init.content,
        }
    }
}
