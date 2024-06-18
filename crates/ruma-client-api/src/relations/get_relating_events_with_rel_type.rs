//! `GET /_matrix/client/*/rooms/{roomId}/relations/{eventId}/{relType}`
//!
//! Get the child events for a given parent event which relate to the parent using the given
//! `rel_type`.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1roomsroomidrelationseventidreltype

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Direction, Metadata},
        metadata,
        serde::Raw,
        OwnedEventId, OwnedRoomId,
    };
    use ruma_events::{relation::RelationType, AnyMessageLikeEvent};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/rooms/{room_id}/relations/{event_id}/{rel_type}",
            1.3 => "/_matrix/client/v1/rooms/{room_id}/relations/{event_id}/{rel_type}",
        }
    };

    /// Request type for the `get_relating_events_with_rel_type` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room containing the parent event.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The ID of the parent event whose child events are to be returned.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// The relationship type to search for.
        #[ruma_api(path)]
        pub rel_type: RelationType,

        /// The pagination token to start returning results from.
        ///
        /// If `None`, results start at the most recent topological event known to the server.
        ///
        /// Can be a `next_batch` token from a previous call, or a returned  `start` token from
        /// `/messages` or a `next_batch` token from `/sync`.
        ///
        /// Note that when paginating the `from` token should be "after" the `to` token in
        /// terms of topological ordering, because it is only possible to paginate "backwards"
        /// through events, starting at `from`.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub from: Option<String>,

        /// The direction to return events from.
        ///
        /// Defaults to [`Direction::Backward`].
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        #[ruma_api(query)]
        pub dir: Direction,

        /// The pagination token to stop returning results at.
        ///
        /// If `None`, results continue up to `limit` or until there are no more events.
        ///
        /// Like `from`, this can be a previous token from a prior call to this endpoint
        /// or from `/messages` or `/sync`.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub to: Option<String>,

        /// The maximum number of results to return in a single `chunk`.
        ///
        /// The server can and should apply a maximum value to this parameter to avoid large
        /// responses.
        ///
        /// Similarly, the server should apply a default value when not supplied.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub limit: Option<UInt>,

        /// Whether to include events which relate indirectly to the given event.
        ///
        /// These are events related to the given event via two or more direct relationships.
        ///
        /// It is recommended that homeservers traverse at least 3 levels of relationships.
        /// Implementations may perform more but should be careful to not infinitely recurse.
        ///
        /// Default to `false`.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        #[ruma_api(query)]
        pub recurse: bool,
    }

    /// Response type for the `get_relating_events_with_rel_type` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The paginated child events which point to the parent.
        ///
        /// The events returned will match the `rel_type` supplied in the URL and are ordered
        /// topologically, most-recent first.
        ///
        /// If no events are related to the parent or the pagination yields no results, an
        /// empty `chunk` is returned.
        pub chunk: Vec<Raw<AnyMessageLikeEvent>>,

        /// An opaque string representing a pagination token.
        ///
        /// If this is `None`, there are no more results to fetch and the client should stop
        /// paginating.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,

        /// An opaque string representing a pagination token.
        ///
        /// If this is `None`, this is the start of the result set, i.e. this is the first
        /// batch/page.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,

        /// If `recurse` was set on the request, the depth to which the server recursed.
        ///
        /// If `recurse` was not set, this field must be absent.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub recursion_depth: Option<UInt>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, parent event ID and relationship type.
        pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, rel_type: RelationType) -> Self {
            Self {
                room_id,
                event_id,
                dir: Direction::default(),
                rel_type,
                from: None,
                to: None,
                limit: None,
                recurse: false,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given chunk.
        pub fn new(chunk: Vec<Raw<AnyMessageLikeEvent>>) -> Self {
            Self { chunk, next_batch: None, prev_batch: None, recursion_depth: None }
        }
    }
}
