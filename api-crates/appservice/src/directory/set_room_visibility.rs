//! [PUT /_matrix/client/r0/directory/list/appservice/{networkId}/{roomId}](https://matrix.org/docs/spec/application_service/r0.1.2#put-matrix-client-r0-directory-list-appservice-networkid-roomid)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Updates the visibility of a given room on the application service's room directory.",
        method: PUT,
        name: "set_room_visibility",
        path: "/_matrix/client/r0/directory/list/appservice/:network_id/:room_id",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The protocol (network) ID to update the room list for.
        #[ruma_api(path)]
        pub network_id: String,

        /// Room ID of the room to add or remove from the directory.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// Whether the room should be visible (public) in the directory or not (private).
        pub visibility: Visibility,
    }

    response: {}
}
