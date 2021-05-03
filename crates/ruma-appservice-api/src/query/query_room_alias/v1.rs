//! [GET /_matrix/app/v1/rooms/{roomAlias}](https://matrix.org/docs/spec/application_service/r0.1.2#get-matrix-app-v1-rooms-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::RoomAliasId;

ruma_api! {
    metadata: {
        description: "This endpoint is invoked by the homeserver on an application service to query the existence of a given room alias.",
        method: GET,
        name: "query_room_alias",
        path: "/_matrix/app/v1/rooms/:room_alias",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The room alias being queried.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,
    }

    #[derive(Default)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room alias.
    pub fn new(room_alias: &'a RoomAliasId) -> Self {
        Self { room_alias }
    }
}

impl Response {
    /// Create an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
