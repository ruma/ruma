//! [GET /_matrix/client/r0/joined_rooms](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-joined-rooms)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Get a list of the user's current rooms.",
        method: GET,
        name: "joined_rooms",
        path: "/_matrix/client/r0/joined_rooms",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of the rooms the user is in, i.e. the ID of each room in
        /// which the user has joined membership.
        pub joined_rooms: Vec<RoomId>,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given joined rooms.
    pub fn new(joined_rooms: Vec<RoomId>) -> Self {
        Self { joined_rooms }
    }
}
