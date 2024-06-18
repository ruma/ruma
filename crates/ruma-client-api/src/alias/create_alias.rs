//! `PUT /_matrix/client/*/directory/room/{roomAlias}`
//!
//! Add an alias to a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3directoryroomroomalias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomAliasId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/directory/room/{room_alias}",
            1.1 => "/_matrix/client/v3/directory/room/{room_alias}",
        }
    };

    /// Request type for the `create_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room alias to set.
        #[ruma_api(path)]
        pub room_alias: OwnedRoomAliasId,

        /// The room ID to set.
        pub room_id: OwnedRoomId,
    }

    /// Response type for the `create_alias` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room alias and room id.
        pub fn new(room_alias: OwnedRoomAliasId, room_id: OwnedRoomId) -> Self {
            Self { room_alias, room_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
