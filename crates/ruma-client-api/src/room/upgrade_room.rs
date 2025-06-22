//! `POST /_matrix/client/*/rooms/{roomId}/upgrade`
//!
//! Upgrades a room to a particular version.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidupgrade

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, RoomVersionId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/upgrade",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/upgrade",
        }
    };

    /// Request type for the `upgrade_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// ID of the room to be upgraded.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// New version for the room.
        pub new_version: RoomVersionId,
    }

    /// Response type for the `upgrade_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// ID of the new room.
        pub replacement_room: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID and new room version.
        pub fn new(room_id: OwnedRoomId, new_version: RoomVersionId) -> Self {
            Self { room_id, new_version }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(replacement_room: OwnedRoomId) -> Self {
            Self { replacement_room }
        }
    }
}
