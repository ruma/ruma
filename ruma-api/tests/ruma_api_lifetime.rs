use ruma_identifiers::{RoomAliasId, RoomId};

mod empty_response {
    use super::*;

    ruma_api::ruma_api! {
        metadata: {
            description: "Add an alias to a room.",
            method: PUT,
            name: "create_alias",
            path: "/_matrix/client/r0/directory/room/:room_alias",
            rate_limited: false,
            requires_authentication: true,
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
    use super::*;

    ruma_api::ruma_api! {
        metadata: {
            description: "Add an alias to a room.",
            method: PUT,
            name: "create_alias",
            path: "/_matrix/client/r0/directory/room/:room_alias",
            rate_limited: false,
            requires_authentication: true,
        }

        request: {
            /// The room alias to set.
            pub room_alias: &'a [Option<&'a RoomAliasId>],

            /// The room ID to set.
            pub room_id: &'b [Option<Option<&'a ruma_identifiers::DeviceId>>],
        }

        response: {}

    }
}

mod full_request_response {
    #[allow(unused)]
    #[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct OtherThing<'t> {
        some: &'t str,
        t: &'t [u8],
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct IncomingOtherThing {
        some: String,
        t: Vec<u8>,
    }

    ruma_api::ruma_api! {
        metadata: {
            description: "Does something.",
            method: GET,
            name: "no_fields",
            path: "/_matrix/my/endpoint/:thing",
            rate_limited: false,
            requires_authentication: false,
        }

        request: {
            #[ruma_api(query)]
            pub abc: &'a str,
            #[ruma_api(path)]
            pub thing: &'a str,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: &'a str,
        }

        response: {
            #[ruma_api(body)]
            pub thing: OtherThing<'a>,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: &'a str,
        }
    }
}

mod full_request_response_with_query_map {
    #[allow(unused)]
    #[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct OtherThing<'t> {
        some: &'t str,
        t: &'t [u8],
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct IncomingOtherThing {
        some: String,
        t: Vec<u8>,
    }

    ruma_api::ruma_api! {
        metadata: {
            description: "Does something.",
            method: GET,
            name: "no_fields",
            path: "/_matrix/my/endpoint/:thing",
            rate_limited: false,
            requires_authentication: false,
        }

        request: {
            #[ruma_api(query_map)]
            // pub abc: &'a [(&'a str, &'a str)],
            pub abc: &'a [(String, String)],
            #[ruma_api(path)]
            pub thing: &'a str,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: &'a str,
        }

        response: {
            #[ruma_api(body)]
            pub thing: OtherThing<'a>,
            #[ruma_api(header = CONTENT_TYPE)]
            pub stuff: &'a str,
        }
    }
}
