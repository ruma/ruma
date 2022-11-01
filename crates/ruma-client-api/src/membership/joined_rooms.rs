//! `GET /_matrix/client/*/joined_rooms`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3joined_rooms

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        description: "Get a list of the user's current rooms.",
        method: GET,
        name: "joined_rooms",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/joined_rooms",
            1.1 => "/_matrix/client/v3/joined_rooms",
        }
    };

    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of the rooms the user is in, i.e. the ID of each room in
        /// which the user has joined membership.
        pub joined_rooms: Vec<OwnedRoomId>,
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
