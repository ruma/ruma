//! `POST /_matrix/client/*/rooms/{roomId}/upgrade`
//!
//! Upgrades a room to a particular version.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidupgrade

    use ruma_common::{
        RoomId, RoomVersionId, UserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/upgrade",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/upgrade",
        }
    }

    /// Request type for the `upgrade_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// A list of user IDs to consider as additional creators, and hence grant an "infinite"
        /// immutable power level, from room version 12 onwards.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub additional_creators: Vec<UserId>,

        /// ID of the room to be upgraded.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// New version for the room.
        pub new_version: RoomVersionId,
    }

    /// Response type for the `upgrade_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// ID of the new room.
        pub replacement_room: RoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID and new room version.
        pub fn new(room_id: RoomId, new_version: RoomVersionId) -> Self {
            Self { room_id, new_version, additional_creators: Vec::new() }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(replacement_room: RoomId) -> Self {
            Self { replacement_room }
        }
    }
}
