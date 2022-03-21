#![allow(clippy::exhaustive_structs)]

#[derive(Copy, Clone, Debug, ruma_common::serde::Incoming, serde::Serialize)]
pub struct OtherThing<'t> {
    pub some: &'t str,
    pub t: &'t [u8],
}

mod empty_response {
    use ruma_common::{api::ruma_api, RoomAliasId, RoomId};

    ruma_api! {
        metadata: {
            description: "Add an alias to a room.",
            method: PUT,
            name: "create_alias",
            unstable_path: "/_matrix/client/r0/directory/room/:room_alias",
            rate_limited: false,
            authentication: AccessToken,
        }

        request: {
            /// The room alias to set.
            #[ruma_api(path)]
            pub room_alias: &'a RoomAliasId,

            /// The room ID to set.
            pub room_id: &'a RoomId,
        }

        response: {}
    }
}

mod nested_types {
    use ruma_common::{api::ruma_api, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "Add an alias to a room.",
            method: PUT,
            name: "create_alias",
            unstable_path: "/_matrix/client/r0/directory/room",
            rate_limited: false,
            authentication: AccessToken,
        }

        request: {
            /// The room alias to set.
            pub room_alias: &'a [Option<&'a RoomAliasId>],

            /// The room ID to set.
            pub room_id: &'b [Option<Option<&'a ruma_common::DeviceId>>],
        }

        response: {}
    }
}

mod full_request_response {
    use ruma_common::api::ruma_api;

    use super::{IncomingOtherThing, OtherThing};

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "no_fields",
            unstable_path: "/_matrix/my/endpoint/:thing",
            rate_limited: false,
            authentication: None,
        }

        request: {
            #[ruma_api(query)]
            pub abc: &'a str,
            #[ruma_api(path)]
            pub thing: &'a str,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: &'a str,
            pub more: OtherThing<'t>,
        }

        response: {
            #[ruma_api(body)]
            pub thing: Vec<String>,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: String,
        }
    }
}

mod full_request_response_with_query_map {
    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: GET,
            name: "no_fields",
            unstable_path: "/_matrix/my/endpoint/:thing",
            rate_limited: false,
            authentication: None,
        }

        request: {
            #[ruma_api(query_map)]
            // pub abc: &'a [(&'a str, &'a str)], // TODO handle this use case
            pub abc: Vec<(String, String)>,
            #[ruma_api(path)]
            pub thing: &'a str,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: &'a str,
        }

        response: {
            #[ruma_api(body)]
            pub thing: String,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: String,
        }
    }
}

mod query_fields {
    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Get the list of rooms in this homeserver's public directory.",
            method: GET,
            name: "get_public_rooms",
            unstable_path: "/_matrix/client/r0/publicRooms",
            rate_limited: false,
            authentication: None,
        }

        request: {
            /// Limit for the number of results to return.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub limit: Option<usize>,

            /// Pagination token from a previous request.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub since: Option<&'a str>,

            /// The server to fetch the public room lists from.
            ///
            /// `None` means the server this request is sent to.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub server: Option<&'a str>,
        }

        response: {}
    }
}
