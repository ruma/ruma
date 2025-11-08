//! `GET /_matrix/federation/*/query/directory`
//!
//! Get mapped room ID and resident homeservers for a given room alias.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1querydirectory

    use ruma_common::{
        OwnedRoomAliasId, OwnedRoomId, OwnedServerName,
        api::{request, response},
        metadata,
    };

    use crate::authentication::ServerSignatures;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/query/directory",
    }

    /// Request type for the `get_room_information` endpoint.
    #[request]
    pub struct Request {
        /// Room alias to query.
        #[ruma_api(query)]
        pub room_alias: OwnedRoomAliasId,
    }

    /// Response type for the `get_room_information` endpoint.
    #[response]
    pub struct Response {
        /// Room ID mapped to queried alias.
        pub room_id: OwnedRoomId,

        /// An array of server names that are likely to hold the given room.
        pub servers: Vec<OwnedServerName>,
    }

    impl Request {
        /// Creates a new `Request` with the given room alias ID.
        pub fn new(room_alias: OwnedRoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room IDs and servers.
        pub fn new(room_id: OwnedRoomId, servers: Vec<OwnedServerName>) -> Self {
            Self { room_id, servers }
        }
    }
}
