//! [GET /_matrix/federation/v1/query/{queryType}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-query-querytype)

use ruma_api::ruma_api;
use serde_json::Value as JsonValue;

use std::collections::BTreeMap;

ruma_api! {
    metadata: {
        description: "Performs a single query request on the receiving homeserver. The query string arguments are dependent on which type of query is being made.",
        method: GET,
        name: "custom",
        path: "/_matrix/federation/v1/query/:query_type",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The type of query to make.
        #[ruma_api(path)]
        pub query_type: &'a str,

        /// The body of the query
        #[ruma_api(query_map)]
        pub body: BTreeMap<String, String>,
    }

    response: {
        /// The body of the response
        #[ruma_api(body)]
        pub body: JsonValue,
    }
}

impl<'a> Request<'a> {
    /// Creates a new request of the given type with the given body
    pub fn new(query_type: &'a str, body: BTreeMap<String, String>) -> Self {
        Self { query_type, body }
    }
}

impl Response {
    /// Creates a new response with the given body
    pub fn new(body: JsonValue) -> Self {
        Self { body }
    }
}
