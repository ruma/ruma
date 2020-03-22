//! [GET /_matrix/client/r0/publicRooms](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-publicrooms)

use js_int::UInt;
use ruma_api::ruma_api;

use super::PublicRoomsChunk;

ruma_api! {
    metadata {
        description: "Get the list of rooms in this homeserver's public directory.",
        method: GET,
        name: "get_public_rooms",
        path: "/_matrix/client/r0/publicRooms",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// Limit for the number of results to return.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub limit: Option<UInt>,
        /// Pagination token from a previous request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<String>,
        /// The server to fetch the public room lists from.
        ///
        /// `None` means the server this request is sent to.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub server: Option<String>,
    }

    response {
        /// A paginated chunk of public rooms.
        pub chunk: Vec<PublicRoomsChunk>,
        /// A pagination token for the response.
        pub next_batch: Option<String>,
        /// A pagination token that allows fetching previous results.
        pub prev_batch: Option<String>,
        /// An estimate on the total number of public rooms, if the server has an estimate.
        pub total_room_count_estimate: Option<UInt>,
    }

    error: crate::Error
}
