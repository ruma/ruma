//! `POST /_matrix/client/*/join/{roomIdOrAlias}`
//!
//! Join a room using its ID or one of its aliases.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3joinroomidoralias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedRoomOrAliasId, OwnedServerName,
    };

    use crate::membership::ThirdPartySigned;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/join/:room_id_or_alias",
            1.1 => "/_matrix/client/v3/join/:room_id_or_alias",
        }
    };

    /// Request type for the `join_room_by_id_or_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// The servers to attempt to join the room through.
        ///
        /// One of the servers  must be participating in the room.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub server_name: Vec<OwnedServerName>,

        /// The signature of a `m.third_party_invite` token to prove that this user owns a third
        /// party identity which has been invited to the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub third_party_signed: Option<ThirdPartySigned>,

        /// Optional reason for joining the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `join_room_by_id_or_alias` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room that the user joined.
        pub room_id: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID or alias ID.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId) -> Self {
            Self { room_id_or_alias, server_name: vec![], third_party_signed: None, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }
}
