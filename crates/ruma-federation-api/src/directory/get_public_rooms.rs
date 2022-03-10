//! `GET /_matrix/federation/*/publicRooms`
//!
//! Endpoint to query a homeserver's public rooms.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#post_matrixfederationv1publicrooms

    use js_int::UInt;
    use ruma_common::{
        api::ruma_api,
        directory::{IncomingRoomNetwork, PublicRoomsChunk, RoomNetwork},
    };

    ruma_api! {
        metadata: {
            description: "Gets all the public rooms for the homeserver.",
            method: GET,
            name: "get_public_rooms",
            stable_path: "/_matrix/federation/v1/publicRooms",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        #[derive(Default)]
        request: {
            /// Limit for the number of results to return.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub limit: Option<UInt>,

            /// Pagination token from a previous request.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub since: Option<&'a str>,

            /// Network to fetch the public room lists from.
            #[serde(flatten, skip_serializing_if = "ruma_common::serde::is_default")]
            #[ruma_api(query)]
            pub room_network: RoomNetwork<'a>,
        }

        #[derive(Default)]
        response: {
            /// A paginated chunk of public rooms.
            pub chunk: Vec<PublicRoomsChunk>,

            /// A pagination token for the response.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub next_batch: Option<String>,

            /// A pagination token that allows fetching previous results.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub prev_batch: Option<String>,

            /// An estimate on the total number of public rooms, if the server has an estimate.
            pub total_room_count_estimate: Option<UInt>,
        }
    }

    impl Request<'_> {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
