//! `GET /_matrix/client/*/joined_rooms`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3joined_rooms

    use ruma_common::{api::ruma_api, OwnedRoomId};

    ruma_api! {
        metadata: {
            description: "Get a list of the user's current rooms.",
            method: GET,
            name: "joined_rooms",
            r0_path: "/_matrix/client/r0/joined_rooms",
            stable_path: "/_matrix/client/v3/joined_rooms",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// A list of the rooms the user is in, i.e. the ID of each room in
            /// which the user has joined membership.
            pub joined_rooms: Vec<OwnedRoomId>,
        }

        error: crate::Error
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given joined rooms.
        pub fn new(joined_rooms: Vec<OwnedRoomId>) -> Self {
            Self { joined_rooms }
        }
    }
}
