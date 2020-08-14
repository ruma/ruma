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
    content: Value,

    /// The `state_key` for the event.
    state_key: String,

    /// The `type` for the event.
    #[serde(rename = "type")]
    kind: EventType,

    /// The `sender` for the event.
    sender: UserId,
}

/// The invite event sent as a response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InviteEvent {
    /// The matrix ID of the user who sent the original `m.room.third_party_invite`.
    sender: UserId,

    /// The name of the inviting homeserver.
    origin: Box<ServerName>,

    /// A timestamp added by the inviting homeserver.
    origin_server_ts: UInt,

    /// The event type (should always be `m.room.member`).
    #[serde(rename = "type")]
    kind: EventType,

    /// The user ID of the invited member.
    state_key: UserId,

    /// The content of the event. Must include a `membership` of invite.
    content: MemberEventContent,
}
