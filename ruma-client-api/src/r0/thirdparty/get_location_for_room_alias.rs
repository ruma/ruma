//! [GET /_matrix/client/r0/thirdparty/location](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-location)

use ruma_api::ruma_api;
use ruma_identifiers::RoomAliasId;

use super::Location;

ruma_api! {
    metadata {
        description: "Retrieve an array of third party network locations from a Matrix room alias.",
        method: GET,
        name: "get_location_for_room_alias",
        path: "/_matrix/client/r0/thirdparty/location",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The Matrix room alias to look up.
        #[ruma_api(query)]
        pub alias: RoomAliasId,
    }

    response {
        /// List of matched third party locations.
        #[ruma_api(body)]
        pub locations: Vec<Location>,
    }

    error: crate::Error
}
