//! [GET /_matrix/client/r0/rooms/{roomId}/joined_members](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-joined-members)

use std::collections::HashMap;

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Get a map of user ids to member info objects for members of the room. Primarily for use in Application Services.",
        method: GET,
        name: "joined_members",
        path: "/_matrix/client/r0/rooms/:room_id/joined_members",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to get the members of.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {
        /// A list of the rooms the user is in, i.e.
        /// the ID of each room in which the user has joined membership.
        pub joined: HashMap<UserId, RoomMember>,
    }

    error: crate::Error
}

// TODO: Find out whether display_name and avatar_url are optional
/// Information about a room member.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomMember {
    /// The display name of the user.
    pub display_name: String,
    /// The mxc avatar url of the user.
    pub avatar_url: Option<String>,
}
