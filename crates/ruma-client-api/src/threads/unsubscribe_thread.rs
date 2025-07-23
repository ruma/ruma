//! `DELETE /_matrix/client/*/rooms/{roomId}/thread/{eventId}/subscription`
//!
//! Removes the subscription state of the current user to a thread in a room.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/blob/rei/msc_thread_subscriptions/proposals/4306-thread-subscriptions.md

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4306") => "/_matrix/client/unstable/io.element.msc4306/rooms/{room_id}/thread/{event_id}/subscription",
        }
    };

    /// Request type for the `unsubscribe_thread` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID where the thread is located.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event ID of the thread root to unsubscribe to.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,
    }

    /// Response type for the `unsubscribe_thread` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` for the given room and thread IDs.
        pub fn new(room_id: OwnedRoomId, thread_root: OwnedEventId) -> Self {
            Self { room_id, event_id: thread_root }
        }
    }

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
