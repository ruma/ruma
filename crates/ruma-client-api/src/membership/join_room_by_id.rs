//! `POST /_matrix/client/*/rooms/{roomId}/join`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidjoin

    use ruma_common::{api::ruma_api, OwnedRoomId, RoomId};

    use crate::membership::{IncomingThirdPartySigned, ThirdPartySigned};

    ruma_api! {
        metadata: {
            description: "Join a room using its ID.",
            method: POST,
            name: "join_room_by_id",
            r0_path: "/_matrix/client/r0/rooms/:room_id/join",
            stable_path: "/_matrix/client/v3/rooms/:room_id/join",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room where the user should be invited.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

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
        /// Creates a new `Request` with the given room id.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id, third_party_signed: None, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }
}
