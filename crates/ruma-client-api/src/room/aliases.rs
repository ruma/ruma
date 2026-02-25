//! `GET /_matrix/client/*/rooms/{roomId}/aliases`
//!
//! Get a list of aliases maintained by the local server for the given room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidaliases

    use ruma_common::{
        RoomAliasId, RoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc2432") => "/_matrix/client/unstable/org.matrix.msc2432/rooms/{room_id}/aliases",
            1.0 => "/_matrix/client/r0/rooms/{room_id}/aliases",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/aliases",
        }
    }

    /// Request type for the `aliases` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room ID to get aliases of.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    /// Response type for the `aliases` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The server's local aliases on the room.
        pub aliases: Vec<RoomAliasId>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: RoomId) -> Self {
            Self { room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given aliases.
        pub fn new(aliases: Vec<RoomAliasId>) -> Self {
            Self { aliases }
        }
    }
}
