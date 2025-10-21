//! `POST /_matrix/federation/*/publicRooms`
//!
//! Get a homeserver's public rooms with an optional filter.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#post_matrixfederationv1publicrooms

    use js_int::UInt;
    use ruma_common::{
        api::{request, response},
        directory::{Filter, PublicRoomsChunk, RoomNetwork},
        metadata,
    };

    use crate::authentication::ServerSignatures;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/publicRooms",
    }

    /// Request type for the `get_public_rooms_filtered` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {
        /// Limit for the number of results to return.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Pagination token from a previous request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub since: Option<String>,

        /// Filter to apply to the results.
        #[serde(default, skip_serializing_if = "Filter::is_empty")]
        pub filter: Filter,

        /// Network to fetch the public room lists from.
        #[serde(flatten, skip_serializing_if = "ruma_common::serde::is_default")]
        pub room_network: RoomNetwork,
    }

    /// Response type for the `get_public_rooms_filtered` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// A paginated chunk of public rooms.
        pub chunk: Vec<PublicRoomsChunk>,

        /// A pagination token for the response.
        pub next_batch: Option<String>,

        /// A pagination token that allows fetching previous results.
        pub prev_batch: Option<String>,

        /// An estimate on the total number of public rooms, if the server has an estimate.
        pub total_room_count_estimate: Option<UInt>,
    }

    impl Request {
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
