//! `PUT /_matrix/client/*/user/{userId}/rooms/{roomId}/account_data/{type}`
//!
//! Associate account data with a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3useruseridroomsroomidaccount_datatype

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId, OwnedUserId,
    };
    use ruma_events::{
        AnyRoomAccountDataEventContent, RoomAccountDataEventContent, RoomAccountDataEventType,
    };
    use serde_json::value::to_raw_value as to_raw_json_value;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/rooms/{room_id}/account_data/{event_type}",
            1.1 => "/_matrix/client/v3/user/{user_id}/rooms/{room_id}/account_data/{event_type}",
        }
    };

    /// Request type for the `set_room_account_data` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the user to set account_data for.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The ID of the room to set account_data on.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

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

    /// Response type for the `set_room_account_data` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given data, event type, room ID and user ID.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T>(
            user_id: OwnedUserId,
            room_id: OwnedRoomId,
            data: &T,
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
            user_id: OwnedUserId,
            room_id: OwnedRoomId,
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
