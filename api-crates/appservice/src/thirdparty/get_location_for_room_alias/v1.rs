//! [GET /_matrix/app/v1/thirdparty/location](https://matrix.org/docs/spec/application_service/r0.1.2#get-matrix-app-v1-thirdparty-location)

use ruma_api::ruma_api;
use ruma_common::thirdparty::Location;
use ruma_identifiers::RoomAliasId;

ruma_api! {
    metadata: {
        description: "Retrieve an array of third party network locations from a Matrix room alias.",
        method: GET,
        name: "get_location_for_room_alias",
        path: "/_matrix/app/v1/thirdparty/location",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The Matrix room alias to look up.
        #[ruma_api(query)]
        pub alias: &'a RoomAliasId,
    }

    response: {
        /// List of matched third party locations.
        #[ruma_api(body)]
        pub locations: Vec<Location>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room alias id.
    pub fn new(alias: &'a RoomAliasId) -> Self {
        Self { alias }
    }
}

impl Response {
    /// Creates a new `Response` with the given locations.
    pub fn new(locations: Vec<Location>) -> Self {
        Self { locations }
    }
}
