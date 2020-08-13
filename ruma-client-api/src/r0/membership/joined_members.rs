//! [GET /_matrix/client/r0/rooms/{roomId}/joined_members](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-joined-members)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get a map of user ids to member info objects for members of the room. Primarily for use in Application Services.",
        method: GET,
        name: "joined_members",
        path: "/_matrix/client/r0/rooms/:room_id/joined_members",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to get the members of.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    response: {
        /// A list of the rooms the user is in, i.e.
        /// the ID of each room in which the user has joined membership.
        pub joined: BTreeMap<UserId, RoomMember>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID.
    pub fn new(room_id: &'a RoomId) -> Self {
        Self { room_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given joined rooms.
    pub fn new(joined: BTreeMap<UserId, RoomMember>) -> Self {
        Self { joined }
    }
}

/// Information about a room member.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomMember {
    /// The display name of the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The mxc avatar url of the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl RoomMember {
    /// Creates an empty `RoomMember`.
    pub fn new() -> Self {
        Default::default()
    }
}
