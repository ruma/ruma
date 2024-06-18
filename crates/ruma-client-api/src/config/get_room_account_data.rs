//! `GET /_matrix/client/*/user/{userId}/rooms/{roomId}/account_data/{type}`
//!
//! Gets account data room for a user for a given room

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3useruseridroomsroomidaccount_datatype

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId, OwnedUserId,
    };
    use ruma_events::{AnyRoomAccountDataEventContent, RoomAccountDataEventType};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/rooms/{room_id}/account_data/{event_type}",
            1.1 => "/_matrix/client/v3/user/{user_id}/rooms/{room_id}/account_data/{event_type}",
        }
    };

    /// Request type for the `get_room_account_data` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// User ID of user for whom to retrieve data.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// Room ID for which to retrieve data.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Type of data to retrieve.
        #[ruma_api(path)]
        pub event_type: RoomAccountDataEventType,
    }

    /// Response type for the `get_room_account_data` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Account data content for the given type.
        ///
        /// Since the inner type of the `Raw` does not implement `Deserialize`, you need to use
        /// [`Raw::deserialize_as`] to deserialize it.
        #[ruma_api(body)]
        pub account_data: Raw<AnyRoomAccountDataEventContent>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID, room ID and event type.
        pub fn new(
            user_id: OwnedUserId,
            room_id: OwnedRoomId,
            event_type: RoomAccountDataEventType,
        ) -> Self {
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
