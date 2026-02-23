//! `GET /_matrix/client/*/directory/room/{roomAlias}`
//!
//! Resolve a room alias to a room ID.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3directoryroomroomalias

    use ruma_common::{
        RoomAliasId, RoomId, ServerName,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/client/r0/directory/room/{room_alias}",
            1.1 => "/_matrix/client/v3/directory/room/{room_alias}",
        }
    }

    /// Request type for the `get_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room alias.
        #[ruma_api(path)]
        pub room_alias: RoomAliasId,
    }

    /// Response type for the `get_alias` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room ID for this room alias.
        pub room_id: RoomId,

        /// A list of servers that are aware of this room ID.
        pub servers: Vec<ServerName>,
    }

    impl Request {
        /// Creates a new `Request` with the given room alias id.
        pub fn new(room_alias: RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id and servers
        pub fn new(room_id: RoomId, servers: Vec<ServerName>) -> Self {
            Self { room_id, servers }
        }
    }
}
