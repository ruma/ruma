//! `PUT /_matrix/client/*/rooms/{roomId}/thread/{eventId}/subscription`
//!
//! Updates the subscription state of the current user to a thread in a room.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4306

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4306") => "/_matrix/client/unstable/io.element.msc4306/rooms/{room_id}/thread/{thread_root}/subscription",
        }
    };

    /// Request type for the `subscribe_thread` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID where the thread is located.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event ID of the thread root to subscribe to.
        #[ruma_api(path)]
        pub thread_root: OwnedEventId,

        /// Whether the subscription was made automatically by a client, not by manual user choice.
        pub automatic: bool,
    }

    /// Response type for the `subscribe_thread` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` for the given room and thread IDs.
        pub fn new(room_id: OwnedRoomId, thread_root: OwnedEventId, automatic: bool) -> Self {
            Self { room_id, thread_root, automatic }
        }
    }

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
