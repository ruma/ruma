//! `GET /_matrix/client/*/rooms/{roomId}/aliases`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3roomsroomidaliases

    use ruma_common::{api::ruma_api, OwnedRoomAliasId, RoomId};

    ruma_api! {
        metadata: {
            description: "Get a list of aliases maintained by the local server for the given room.",
            method: GET,
            name: "aliases",
            r0_path: "/_matrix/client/r0/rooms/:room_id/aliases",
            stable_path: "/_matrix/client/v3/rooms/:room_id/aliases",
            unstable_path: "/_matrix/client/unstable/org.matrix.msc2432/rooms/:room_id/aliases",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room ID to get aliases of.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
        }

        response: {
            /// The server's local aliases on the room.
            pub aliases: Vec<OwnedRoomAliasId>,
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
        /// Creates a new `Response` with the given aliases.
        pub fn new(aliases: Vec<OwnedRoomAliasId>) -> Self {
            Self { aliases }
        }
    }
}
