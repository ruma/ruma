//! `GET /_matrix/federation/*/publicRooms`
//!
//! Get all the public rooms for the homeserver.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/server-server-api/#post_matrixfederationv1publicrooms

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        directory::{IncomingRoomNetwork, PublicRoomsChunk, RoomNetwork},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/publicRooms",
        }
    };

    /// Request type for the `get_public_rooms` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request<'a> {
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

    /// Response type for the `get_public_rooms` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
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
