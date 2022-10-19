//! `PUT /_matrix/client/*/user/{userId}/rooms/{roomId}/account_data/{type}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3useruseridroomsroomidaccount_datatype

    use ruma_common::{
        api::ruma_api,
        events::{
            AnyRoomAccountDataEventContent, RoomAccountDataEventContent, RoomAccountDataEventType,
        },
        serde::Raw,
        RoomId, UserId,
    };
    use serde_json::value::to_raw_value as to_raw_json_value;

    ruma_api! {
        metadata: {
            description: "Associate account data with a room.",
            method: PUT,
            name: "set_room_account_data",
            r0_path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:event_type",
            stable_path: "/_matrix/client/v3/user/:user_id/rooms/:room_id/account_data/:event_type",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The ID of the user to set account_data for.
            ///
            /// The access token must be authorized to make requests for this user ID.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The ID of the room to set account_data on.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The event type of the account_data to set.
            ///
            /// Custom types should be namespaced to avoid clashes.
            #[ruma_api(path)]
            pub event_type: RoomAccountDataEventType,

            /// Arbitrary JSON to store as config data.
            ///
            /// To create a `RawJsonValue`, use `serde_json::value::to_raw_value`.
            #[ruma_api(body)]
            pub data: Raw<AnyRoomAccountDataEventContent>,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given data, event type, room ID and user ID.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T>(
            user_id: &'a UserId,
            room_id: &'a RoomId,
            data: &'a T,
        ) -> serde_json::Result<Self>
        where
            T: RoomAccountDataEventContent,
        {
            Ok(Self {
                user_id,
                room_id,
                event_type: data.event_type(),
                data: Raw::from_json(to_raw_json_value(data)?),
            })
        }

        /// Creates a new `Request` with the given raw data, event type, room ID and user ID.
        pub fn new_raw(
            user_id: &'a UserId,
            room_id: &'a RoomId,
            event_type: RoomAccountDataEventType,
            data: Raw<AnyRoomAccountDataEventContent>,
        ) -> Self {
            Self { user_id, room_id, event_type, data }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
