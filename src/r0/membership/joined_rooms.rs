//! [GET /_matrix/client/r0/joined_rooms](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-joined-rooms)

use ruma_api_macros::ruma_api;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Get a list of the user's current rooms.",
        method: GET,
        name: "joined_rooms",
        path: "/_matrix/client/r0/joined_rooms",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        /// A list of the rooms the user is in, i.e.
        /// the ID of each room in which the user has joined membership.
        pub joined_rooms: Vec<RoomId>,
    }
}
