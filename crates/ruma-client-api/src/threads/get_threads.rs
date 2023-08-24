//! `GET /_matrix/client/*/rooms/{roomId}/threads`
//!
//! Retrieve a list of threads in a room, with optional filters.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1roomsroomidthreads

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::{Raw, StringEnum},
        OwnedRoomId,
    };
    use ruma_events::AnyTimelineEvent;

    use crate::PrivOwnedStr;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3856/rooms/:room_id/threads",
            1.4 => "/_matrix/client/v1/rooms/:room_id/threads",
        }
    };

    /// Request type for the `get_thread_roots` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID where the thread roots are located.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The pagination token to start returning results from.
        ///
        /// If `None`, results start at the most recent topological event visible to the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub from: Option<String>,

        /// Which thread roots are of interest to the caller.
        #[serde(skip_serializing_if = "ruma_common::serde::is_default")]
        #[ruma_api(query)]
        pub include: IncludeThreads,

        /// The maximum number of results to return in a single `chunk`.
        ///
        /// Servers should apply a default value, and impose a maximum value to avoid resource
        /// exhaustion.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub limit: Option<UInt>,
    }

    /// Response type for the `get_thread_roots` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The thread roots, ordered by the `latest_event` in each event's aggregation bundle.
        ///
        /// All events returned include bundled aggregations.
        pub chunk: Vec<Raw<AnyTimelineEvent>>,

        /// An opaque string to provide to `from` to keep paginating the responses.
        ///
        /// If this is `None`, there are no more results to fetch and the client should stop
        /// paginating.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id, from: None, include: IncludeThreads::default(), limit: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given chunk.
        pub fn new(chunk: Vec<Raw<AnyTimelineEvent>>) -> Self {
            Self { chunk, next_batch: None }
        }
    }

    /// Which threads to include in the response.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
    #[ruma_enum(rename_all = "lowercase")]
    #[non_exhaustive]
    pub enum IncludeThreads {
        /// `all`
        ///
        /// Include all thread roots found in the room.
        ///
        /// This is the default.
        #[default]
        All,

        /// `participated`
        ///
        /// Only include thread roots for threads where [`current_user_participated`] is `true`.
        ///
        /// [`current_user_participated`]: https://spec.matrix.org/latest/client-server-api/#server-side-aggregation-of-mthread-relationships
        Participated,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}
