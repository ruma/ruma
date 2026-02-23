//! `DELETE /_matrix/client/*/directory/room/{roomAlias}`
//!
//! Remove an alias from a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#delete_matrixclientv3directoryroomroomalias

    use ruma_common::{
        RoomAliasId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: DELETE,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/directory/room/{room_alias}",
            1.1 => "/_matrix/client/v3/directory/room/{room_alias}",
        }
    }

    /// Request type for the `delete_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room alias to remove.
        #[ruma_api(path)]
        pub room_alias: RoomAliasId,
    }

    /// Response type for the `delete_alias` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room alias.
        pub fn new(room_alias: RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
