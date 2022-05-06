//! `POST /_matrix/client/*/join/{roomIdOrAlias}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3joinroomidoralias

    use ruma_common::{api::ruma_api, OwnedRoomId, OwnedServerName, RoomOrAliasId};

    use crate::membership::{IncomingThirdPartySigned, ThirdPartySigned};

    ruma_api! {
        metadata: {
            description: "Join a room using its ID or one of its aliases.",
            method: POST,
            name: "join_room_by_id_or_alias",
            r0_path: "/_matrix/client/r0/join/:room_id_or_alias",
            stable_path: "/_matrix/client/v3/join/:room_id_or_alias",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room where the user should be invited.
            #[ruma_api(path)]
            pub room_id_or_alias: &'a RoomOrAliasId,

            /// The servers to attempt to join the room through.
            ///
            /// One of the servers  must be participating in the room.
            #[ruma_api(query)]
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub server_name: &'a [OwnedServerName],

            /// The signature of a `m.third_party_invite` token to prove that this user owns a third
            /// party identity which has been invited to the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub third_party_signed: Option<ThirdPartySigned<'a>>,

            /// Optional reason for joining the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub reason: Option<&'a str>,
        }

        response: {
            /// The room that the user joined.
            pub room_id: OwnedRoomId,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID or alias ID.
        pub fn new(room_id_or_alias: &'a RoomOrAliasId) -> Self {
            Self { room_id_or_alias, server_name: &[], third_party_signed: None, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }
}
