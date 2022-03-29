//! `GET /_matrix/client/*/directory/list/room/{roomId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3directorylistroomroomid

    use ruma_common::{api::ruma_api, RoomId};

    use crate::room::Visibility;

    ruma_api! {
        metadata: {
            description: "Get the visibility of a public room on a directory.",
            name: "get_room_visibility",
            method: GET,
            r0_path: "/_matrix/client/r0/directory/list/room/:room_id",
            stable_path: "/_matrix/client/v3/directory/list/room/:room_id",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The ID of the room of which to request the visibility.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
        }

        response: {
            /// Visibility of the room.
            pub visibility: Visibility,
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
        /// Creates a new `Response` with the given visibility.
        pub fn new(visibility: Visibility) -> Self {
            Self { visibility }
        }
    }
}
