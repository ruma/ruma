//! Endpoints for client configuration.

/// [PUT /_matrix/client/r0/user/{userId}/rooms/{roomId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-user-userid-rooms-roomid-account-data-type)
pub mod set_room_account_data {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, UserId};
    use serde_json::Value;

    ruma_api! {
        metadata {
            description: "Associate account data with a room.",
            method: Method::Put,
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
    }
}

/// [PUT /_matrix/client/r0/user/{userId}/account_data/{type}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-user-userid-account-data-type)
pub mod set_global_account_data  {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;
    use serde_json::Value;

    ruma_api! {
        metadata {
            description: "Sets global account data.",
            method: Method::Put,
            name: "set_global_account_data",
            path: "/_matrix/client/r0/user/:user_id/account_data/:event_type",
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
            /// The ID of the user to set account_data for.
            ///
            /// The access token must be authorized to make requests for this user ID.
            #[ruma_api(path)]
            pub user_id: UserId,
        }

        response {}
    }
}
