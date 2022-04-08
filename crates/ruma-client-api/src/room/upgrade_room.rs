//! `POST /_matrix/client/*/rooms/{roomId}/upgrade`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidupgrade

    use ruma_common::{api::ruma_api, OwnedRoomId, RoomId, RoomVersionId};

    ruma_api! {
        metadata: {
            description: "Upgrades a room to a particular version.",
            method: POST,
            name: "upgrade_room",
            r0_path: "/_matrix/client/r0/rooms/:room_id/upgrade",
            stable_path: "/_matrix/client/v3/rooms/:room_id/upgrade",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// ID of the room to be upgraded.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// New version for the room.
            pub new_version: &'a RoomVersionId,
        }

        response: {
            /// ID of the new room.
            pub replacement_room: OwnedRoomId,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and new room version.
        pub fn new(room_id: &'a RoomId, new_version: &'a RoomVersionId) -> Self {
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
