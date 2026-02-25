//! `DELETE /_matrix/client/*/rooms/{roomId}/thread/{eventId}/subscription`
//!
//! Removes the subscription state of the current user to a thread in a room.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4306

    use ruma_common::{
        EventId, RoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4306") => "/_matrix/client/unstable/io.element.msc4306/rooms/{room_id}/thread/{thread_root}/subscription",
        }
    }

    /// Request type for the `unsubscribe_thread` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID where the thread is located.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The event ID of the thread root to unsubscribe to.
        #[ruma_api(path)]
        pub thread_root: EventId,
    }

    /// Response type for the `unsubscribe_thread` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` for the given room and thread IDs.
        pub fn new(room_id: RoomId, thread_root: EventId) -> Self {
            Self { room_id, thread_root }
        }
    }

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
