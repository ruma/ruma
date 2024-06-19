//! `GET /_matrix/client/*/rooms/{roomId}/hierarchy`
//!
//! Paginates over the space tree in a depth-first manner to locate child rooms of a given space.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1roomsroomidhierarchy

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::space::SpaceHierarchyRoomsChunk;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc2946/rooms/{room_id}/hierarchy",
            1.2 => "/_matrix/client/v1/rooms/{room_id}/hierarchy",
        }
    };

    /// Request type for the `hierarchy` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID of the space to get a hierarchy for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// A pagination token from a previous result.
        ///
        /// If specified, `max_depth` and `suggested_only` cannot be changed from the first
        /// request.
        #[ruma_api(query)]
        pub from: Option<String>,

        /// The maximum number of rooms to include per response.
        #[ruma_api(query)]
        pub limit: Option<UInt>,

        /// How far to go into the space.
        ///
        /// When reached, no further child rooms will be returned.
        #[ruma_api(query)]
        pub max_depth: Option<UInt>,

        /// Whether or not the server should only consider suggested rooms.
        ///
        /// Suggested rooms are annotated in their `m.space.child` event contents.
        ///
        /// Defaults to `false`.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub suggested_only: bool,
    }

    /// Response type for the `hierarchy` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// A token to supply to from to keep paginating the responses.
        ///
        /// Not present when there are no further results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,

        /// A paginated chunk of the space children.
        pub rooms: Vec<SpaceHierarchyRoomsChunk>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id, from: None, limit: None, max_depth: None, suggested_only: false }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
