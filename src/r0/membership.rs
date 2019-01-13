//! Endpoints for room membership.

use ruma_signatures::Signatures;
use serde_derive::{Deserialize, Serialize};

// TODO: spec requires a nesting ThirdPartySigned { signed: Signed { mxid: ..., ... } }
//       for join_room_by_id_or_alias but not for join_room_by_id, inconsistency?

/// A signature of an `m.third_party_invite` token to prove that this user owns a third party
/// identity which has been invited to the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThirdPartySigned {
    /// The Matrix ID of the invitee.
    pub mxid: String,
    /// The Matrix ID of the user who issued the invite.
    pub sender: String,
    /// A signatures object containing a signature of the entire signed object.
    pub signatures: Signatures,
    /// The state key of the m.third_party_invite event.
    pub token: String,
}

/// [POST /_matrix/client/r0/rooms/{roomId}/invite](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-invite)
pub mod invite_user {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, UserId};
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Invite a user to a room.",
            method: POST,
            name: "invite_user",
            path: "/_matrix/client/r0/rooms/:room_id/invite",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// The room where the user should be invited.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The user to invite.
            pub user_id: UserId,
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/join/{roomIdOrAlias}](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-join-roomidoralias)
pub mod join_room_by_id_or_alias {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, RoomIdOrAliasId};
    use serde_derive::{Deserialize, Serialize};

    use super::ThirdPartySigned;

    ruma_api! {
        metadata {
            description: "Join a room using its ID or one of its aliases.",
            method: POST,
            name: "join_room_by_id_or_alias",
            path: "/_matrix/client/r0/join/:room_id_or_alias",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// The room where the user should be invited.
            #[ruma_api(path)]
            pub room_id_or_alias: RoomIdOrAliasId,
            /// The signature of a `m.third_party_invite` token to prove that this user owns a third
            /// party identity which has been invited to the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub third_party_signed: Option<ThirdPartySigned>,
        }

        response {
            /// The room that the user joined.
            pub room_id: RoomId,
        }
    }
}

/// [POST /_matrix/client/r0/rooms/{roomId}/join](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-join)
pub mod join_room_by_id {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use serde_derive::{Deserialize, Serialize};

    use super::ThirdPartySigned;

    ruma_api! {
        metadata {
            description: "Join a room using its ID.",
            method: POST,
            name: "join_room_by_id",
            path: "/_matrix/client/r0/rooms/:room_id/join",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// The room where the user should be invited.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The signature of a `m.third_party_invite` token to prove that this user owns a third
            /// party identity which has been invited to the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub third_party_signed: Option<ThirdPartySigned>,
        }

        response {
            /// The room that the user joined.
            pub room_id: RoomId,
        }
    }
}

/// [POST /_matrix/client/r0/rooms/{roomId}/forget](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-forget)
pub mod forget_room {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Forget a room.",
            method: POST,
            name: "forget_room",
            path: "/_matrix/client/r0/rooms/:room_id/forget",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// The room to forget.
            #[ruma_api(path)]
            pub room_id: RoomId,
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/rooms/{roomId}/leave](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-leave)
pub mod leave_room {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::RoomId;
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Leave a room.",
            method: POST,
            name: "leave_room",
            path: "/_matrix/client/r0/rooms/:room_id/leave",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// The room to leave.
            #[ruma_api(path)]
            pub room_id: RoomId,
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/rooms/{roomId}/kick](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-kick)
pub mod kick_user {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, UserId};
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Kick a user from a room.",
            method: POST,
            name: "kick_user",
            path: "/_matrix/client/r0/rooms/:room_id/kick",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The reason for kicking the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub reason: Option<String>,
            /// The room to kick the user from.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The user to kick.
            pub user_id: UserId,
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/rooms/{roomId}/unban](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-unban)
pub mod unban_user {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, UserId};
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Unban a user from a room.",
            method: POST,
            name: "unban_user",
            path: "/_matrix/client/r0/rooms/:room_id/unban",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room to unban the user from.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The user to unban.
            pub user_id: UserId,
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/rooms/{roomId}/ban](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-ban)
pub mod ban_user {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, UserId};
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Ban a user from a room.",
            method: POST,
            name: "ban_user",
            path: "/_matrix/client/r0/rooms/:room_id/ban",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The reason for banning the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub reason: Option<String>,
            /// The room to kick the user from.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The user to ban.
            pub user_id: UserId,
        }

        response {}
    }
}
