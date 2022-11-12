//! `GET /_matrix/client/*/directory/room/{roomAlias}`
//!
//! Resolve a room alias to a room ID.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3directoryroomroomalias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedServerName, RoomAliasId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/directory/room/:room_alias",
            1.1 => "/_matrix/client/v3/directory/room/:room_alias",
        }
    };

    /// Request type for the `get_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The room alias.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,
    }

    /// Response type for the `get_alias` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room ID for this room alias.
        pub room_id: OwnedRoomId,

        /// A list of servers that are aware of this room ID.
        pub servers: Vec<OwnedServerName>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias id.
        pub fn new(room_alias: &'a RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id and servers
        pub fn new(room_id: OwnedRoomId, servers: Vec<OwnedServerName>) -> Self {
            Self { room_id, servers }
        }
    }
}
