//! [GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-rooms-roomid-account-data-type)

use ruma_api::ruma_api;
use ruma_events::{AnyBasicEvent, EventJson};
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Gets account data room for a user for a given room",
        name: "get_room_account_data",
        method: GET,
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/account_data/:event_type",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// User ID of user for whom to retrieve data.
        #[ruma_api(path)]
        pub user_id: UserId,

        /// Room ID for which to retrieve data.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// Type of data to retrieve.
        #[ruma_api(path)]
        pub event_type: String,
    }

    response: {
        /// Account data content for the given type.
        #[ruma_api(body)]
        pub account_data: EventJson<AnyBasicEvent>,
    }

    error: crate::Error
}
