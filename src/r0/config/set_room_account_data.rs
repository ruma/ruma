//! [PUT /_matrix/client/r0/user/{userId}/rooms/{roomId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-user-userid-rooms-roomid-account-data-type)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};
use serde_json::Value;

ruma_api! {
    metadata {
        description: "Associate account data with a room.",
        method: PUT,
        name: "set_room_account_data",
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:event_type",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// Arbitrary JSON to store as config data.
        #[ruma_api(body)]
        pub data: Value,
        /// The event type of the account_data to set.
        ///
        /// Custom types should be namespaced to avoid clashes.
        #[ruma_api(path)]
        pub event_type: String,
        /// The ID of the room to set account_data on.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The ID of the user to set account_data for.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
