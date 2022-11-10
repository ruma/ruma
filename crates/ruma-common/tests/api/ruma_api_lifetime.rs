#![allow(clippy::exhaustive_structs)]

#[derive(Copy, Clone, Debug, ruma_common::serde::Incoming, serde::Serialize)]
pub struct OtherThing<'t> {
    pub some: &'t str,
    pub t: &'t [u8],
}

mod empty_response {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomAliasId, RoomId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/r0/directory/room/:room_alias",
        }
    };

    /// Request type for the `create_alias` endpoint.
    #[request]
    pub struct Request<'a> {
        /// The room alias to set.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,

        /// The room ID to set.
        pub room_id: &'a RoomId,
    }

    /// Response type for the `create_alias` endpoint.
    #[response]
    pub struct Response {}
}

mod nested_types {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomAliasId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/r0/directory/room",
        }
    };

    /// Request type for the `create_alias` endpoint.
    #[request]
    pub struct Request<'a> {
        /// The room alias to set.
        pub room_alias: &'a [Option<&'a RoomAliasId>],

        /// The room ID to set.
        pub room_id: &'a [Option<Option<&'a ruma_common::DeviceId>>],
    }

    /// Response type for the `create_alias` endpoint.
    #[response]
    pub struct Response {}
}

mod full_request_response {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use super::{IncomingOtherThing, OtherThing};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/my/endpoint/:thing",
        }
    };

    /// Request type for the `no_fields` endpoint.
    #[request]
    pub struct Request<'a> {
        #[ruma_api(query)]
        pub abc: &'a str,
        #[ruma_api(path)]
        pub thing: &'a str,
        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: &'a str,
        pub more: OtherThing<'a>,
    }

    /// Response type for the `no_fields` endpoint.
    #[response]
    pub struct Response {
        #[ruma_api(body)]
        pub thing: Vec<String>,
        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: String,
    }
}

mod full_request_response_with_query_map {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/my/endpoint/:thing",
        }
    };

    /// Request type for the `no_fields` endpoint.
    #[request]
    pub struct Request<'a> {
        #[ruma_api(query_map)]
        // pub abc: &'a [(&'a str, &'a str)], // TODO handle this use case
        pub abc: Vec<(String, String)>,
        #[ruma_api(path)]
        pub thing: &'a str,
        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: &'a str,
    }

    /// Response type for the `no_fields` endpoint.
    #[response]
    pub struct Response {
        #[ruma_api(body)]
        pub thing: String,
        #[ruma_api(header = CONTENT_TYPE)]
        pub stuff: String,
    }
}

mod query_fields {
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/client/r0/publicRooms",
        }
    };

    /// Request type for the `get_public_rooms` endpoint.
    #[request]
    pub struct Request<'a> {
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

    /// Response type for the `get_public_rooms` endpoint.
    #[response]
    pub struct Response {}
}
