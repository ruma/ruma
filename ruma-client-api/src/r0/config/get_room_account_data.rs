//! [GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-rooms-roomid-account-data-type)

use ruma_api::ruma_api;
use ruma_events::AnyBasicEvent;
use ruma_identifiers::{RoomId, UserId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Gets account data room for a user for a given room",
        name: "get_room_account_data",
        method: GET,
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:event_type",
        rate_limited: false,
        authentication: AccessToken,
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
        #[ruma_api(body)]
        pub account_data: Raw<AnyBasicEvent>,
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
    pub fn new(account_data: Raw<AnyBasicEvent>) -> Self {
        Self { account_data }
    }
}
