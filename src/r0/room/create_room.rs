//! [POST /_matrix/client/r0/createRoom](https://matrix.org/docs/spec/client_server/r0.6.0.html#post-matrix-client-r0-createroom)

use ruma_api::ruma_api;
use ruma_events::{room::power_levels::PowerLevelsEventContent, EventResult};
use ruma_identifiers::{RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Visibility;
use crate::r0::membership::Invite3pid;

ruma_api! {
    metadata {
        description: "Create a new room.",
        method: POST,
        name: "create_room",
        path: "/_matrix/client/r0/createRoom",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// Extra keys to be added to the content of the `m.room.create`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub creation_content: Option<CreationContent>,
        /// List of state events to send to the new room.
        ///
        /// Takes precedence over events set by preset, but gets overriden by
        /// name and topic keys.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub initial_state: Vec<InitialStateEvent>,
        /// A list of user IDs to invite to the room.
        ///
        /// This will tell the server to invite everyone in the list to the newly created room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub invite: Vec<UserId>,
        /// List of third party IDs of users to invite.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub invite_3pid: Vec<Invite3pid>,
        /// If set, this sets the `is_direct` flag on room invites.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_direct: Option<bool>,
        /// If this is included, an `m.room.name` event will be sent into the room to indicate
        /// the name of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// Power level content to override in the default power level event.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[wrap_incoming(PowerLevelsEventContent with EventResult)]
        pub power_level_content_override: Option<PowerLevelsEventContent>,
        /// Convenience parameter for setting various default state events based on a preset.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub preset: Option<RoomPreset>,
        /// The desired room alias local part.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_alias_name: Option<String>,
        /// Room version to set for the room. Defaults to homeserver's default if not specified.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_version: Option<String>,
        /// If this is included, an `m.room.topic` event will be sent into the room to indicate
        /// the topic for the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic: Option<String>,
        /// A public visibility indicates that the room will be shown in the published room
        /// list. A private visibility will hide the room from the published room list. Rooms
        /// default to private visibility if this key is not included.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub visibility: Option<Visibility>,
    }

    response {
        /// The created room's ID.
        pub room_id: RoomId,
    }

    error: crate::Error
}

/// Extra options to be added to the `m.room.create` event.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct CreationContent {
    /// Whether users on other servers can join this room.
    ///
    /// Defaults to `true` if key does not exist.
    #[serde(rename = "m.federate", skip_serializing_if = "Option::is_none")]
    pub federate: Option<bool>,
}

/// A convenience parameter for setting a few default state events.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomPreset {
    /// `join_rules` is set to `invite` and `history_visibility` is set to `shared`.
    PrivateChat,
    /// `join_rules` is set to `public` and `history_visibility` is set to `shared`.
    PublicChat,
    /// Same as `PrivateChat`, but all initial invitees get the same power level as the creator.
    TrustedPrivateChat,
}

/// Represents content of a state event to be used to initalize new room state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitialStateEvent {
    /// State event type.
    #[serde(rename = "type")]
    pub event_type: String,
    /// `state_key` of the event to be sent.
    pub state_key: Option<String>,
    /// JSON content of the state event.
    pub content: Value,
}
