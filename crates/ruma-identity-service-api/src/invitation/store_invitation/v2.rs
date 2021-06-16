//! [POST /_matrix/identity/v2/store-invite](https://matrix.org/docs/spec/identity_service/r0.3.0#post-matrix-identity-v2-store-invite)

use ruma_api::ruma_api;
use ruma_common::thirdparty::Medium;
use ruma_identifiers::{MxcUri, RoomAliasId, RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Store pending invitations to a user's 3PID.",
        method: POST,
        name: "store_invitation",
        path: "/_matrix/identity/v2/store-invite",
        authentication: AccessToken,
        rate_limited: false,
    }

    request: {
        /// The literal string `email`.
        pub medium: &'a Medium,

        /// The email address of the invited user.
        pub address: &'a str,

        /// The Matrix room ID to which the user is invited.
        pub room_id: &'a RoomId,

        /// The Matrix user ID of the inviting user.
        pub sender: &'a UserId,

        /// The Matrix room alias for the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.canonical` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_alias: Option<&'a RoomAliasId>,

        /// The Content URI for the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.avatar` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_avatar_url: Option<&'a MxcUri>,

        /// The `join_rule` for the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.join_rules` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_join_rules: Option<&'a str>,

        /// The name of the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.name` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_name: Option<&'a str>,

        /// The display name of the user ID initiating the invite.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sender_display_name: Option<&'a str>,

        /// The Content URI for the avater of the user ID initiating the invite.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sender_avatar_url: Option<&'a MxcUri>,
    }

    response: {
        /// The generated token. Must be a string consisting of the characters `[0-9a-zA-Z.=_-]`.
        ///
        /// Its length must not exceed 255 characters and it must not be empty.
        pub token: String,

        /// A list of [server's long-term public key, generated ephemeral public key].
        pub public_keys: Vec<String>,

        /// The generated (redacted) display_name. An example is `f...@b...`.
        pub display_name: String,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given address, room_id and sender.
    pub fn new(address: &'a str, room_id: &'a RoomId, sender: &'a UserId) -> Self {
        Self {
            medium: &Medium::Email,
            address,
            room_id,
            sender,
            room_alias: None,
            room_avatar_url: None,
            room_join_rules: None,
            room_name: None,
            sender_display_name: None,
            sender_avatar_url: None,
        }
    }
}

impl Response {
    /// Creates a new `Response` with the given token, public keys and display name.
    pub fn new(token: String, public_keys: Vec<String>, display_name: String) -> Self {
        Self { token, public_keys, display_name }
    }
}
