//! `PUT /_matrix/client/*/directory/list/room/{roomId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3directorylistroomroomid

    use ruma_common::{api::ruma_api, RoomId};

    use crate::room::Visibility;

    ruma_api! {
        metadata: {
            description: "Set the visibility of a public room on a directory.",
            name: "set_room_visibility",
            method: PUT,
            r0_path: "/_matrix/client/r0/directory/list/room/:room_id",
            stable_path: "/_matrix/client/v3/directory/list/room/:room_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The ID of the room of which to set the visibility.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// New visibility setting for the room.
            pub visibility: Visibility,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and visibility.
        pub fn new(room_id: &'a RoomId, visibility: Visibility) -> Self {
            Self { room_id, visibility }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
