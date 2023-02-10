//! `POST /_matrix/client/*/knock/{roomIdOrAlias}`
//!
//! Knock on a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3knockroomidoralias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedRoomOrAliasId, OwnedServerName,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/xyz.amorgan.knock/knock/:room_id_or_alias",
            1.1 => "/_matrix/client/v3/knock/:room_id_or_alias",
        }
    };

    /// Request type for the `knock_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room the user should knock on.
        #[ruma_api(path)]
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// The reason for joining a room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,

        /// The servers to attempt to knock on the room through.
        ///
        /// One of the servers must be participating in the room.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub server_name: Vec<OwnedServerName>,
    }

    /// Response type for the `knock_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room that the user knocked on.
        pub room_id: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID or alias.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId) -> Self {
            Self { room_id_or_alias, reason: None, server_name: vec![] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }
}
