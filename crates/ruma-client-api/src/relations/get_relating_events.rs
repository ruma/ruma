//! `GET /_matrix/client/*/rooms/{roomId}/relations/{eventId}`
//!
//! Retrieve all of the child events for a given parent event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv1roomsroomidrelationseventid

    use js_int::UInt;
    use ruma_common::{api::ruma_api, events::AnyMessageLikeEvent, serde::Raw, EventId, RoomId};

    use crate::Direction;

    ruma_api! {
        metadata: {
            description: "Get the child events for a given parent event.",
            method: GET,
            name: "get_relating_events",
            unstable_path: "/_matrix/client/unstable/rooms/:room_id/relations/:event_id",
            stable_path: "/_matrix/client/v1/rooms/:room_id/relations/:event_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.3,
        }

        request: {
            /// The ID of the room containing the parent event.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The ID of the parent event whose child events are to be returned.
            #[ruma_api(path)]
            pub event_id: &'a EventId,

            /// The pagination token to start returning results from.
            ///
            /// If `None`, results start at the most recent topological event known to the server.
            ///
            /// Can be a `next_batch` or `prev_batch` token from a previous call, or a returned
            /// `start` token from `/messages` or a `next_batch` token from `/sync`.
            ///
            /// Note that when paginating the `from` token should be "after" the `to` token in
            /// terms of topological ordering, because it is only possible to paginate "backwards"
            /// through events, starting at `from`.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub from: Option<&'a str>,

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
            pub to: Option<&'a str>,

            /// The maximum number of results to return in a single `chunk`.
            ///
            /// The server can and should apply a maximum value to this parameter to avoid large
            /// responses.
            ///
            /// Similarly, the server should apply a default value when not supplied.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub limit: Option<UInt>,
        }

        response: {
            /// The paginated child events which point to the parent.
            ///
            /// The events returned are ordered topologically, most-recent first.
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
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and parent event ID.
        pub fn new(room_id: &'a RoomId, event_id: &'a EventId) -> Self {
            Self { room_id, event_id, dir: Direction::default(), from: None, to: None, limit: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given chunk.
        pub fn new(chunk: Vec<Raw<AnyMessageLikeEvent>>) -> Self {
            Self { chunk, next_batch: None, prev_batch: None }
        }
    }
}
