//! `GET /_matrix/client/*/user/{userId}/rooms/{roomId}/account_data/{type}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3useruseridroomsroomidaccount_datatype

    use ruma_common::{
        api::ruma_api, events::AnyRoomAccountDataEventContent, serde::Raw, RoomId, UserId,
    };

    ruma_api! {
        metadata: {
            description: "Gets account data room for a user for a given room",
            name: "get_room_account_data",
            method: GET,
            r0_path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:event_type",
            stable_path: "/_matrix/client/v3/user/:user_id/rooms/:room_id/account_data/:event_type",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// User ID of user for whom to retrieve data.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// Room ID for which to retrieve data.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// Type of data to retrieve.
            #[ruma_api(path)]
            pub event_type: &'a str,
        }

        response: {
            /// Account data content for the given type.
            ///
            /// Use [`Raw::deserialize_content`] for deserialization.
            #[ruma_api(body)]
            pub account_data: Raw<AnyRoomAccountDataEventContent>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID, room ID and event type.
        pub fn new(user_id: &'a UserId, room_id: &'a RoomId, event_type: &'a str) -> Self {
            Self { user_id, room_id, event_type }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given account data.
        pub fn new(account_data: Raw<AnyRoomAccountDataEventContent>) -> Self {
            Self { account_data }
        }
    }
}
