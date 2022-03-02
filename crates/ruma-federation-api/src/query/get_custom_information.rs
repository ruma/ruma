//! `GET /_matrix/federation/*/query/{queryType}`
//!
//! Generic query endpoint for performing custom queries.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1queryquerytype

    use std::collections::BTreeMap;

    use ruma_common::api::ruma_api;
    use serde_json::Value as JsonValue;

    ruma_api! {
        metadata: {
            description: "Performs a single query request on the receiving homeserver. The query string arguments are dependent on which type of query is being made.",
            method: GET,
            name: "get_custom_information",
            stable_path: "/_matrix/federation/v1/query/:query_type",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The type of query to make.
            #[ruma_api(path)]
            pub query_type: &'a str,

            /// The query parameters.
            #[ruma_api(query_map)]
            pub params: BTreeMap<String, String>,
        }

        response: {
            /// The body of the response.
            #[ruma_api(body)]
            pub body: JsonValue,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new request with the given type and query parameters.
        pub fn new(query_type: &'a str, params: BTreeMap<String, String>) -> Self {
            Self { query_type, params }
        }
    }

    impl Response {
        /// Creates a new response with the given body.
        pub fn new(body: JsonValue) -> Self {
            Self { body }
        }
    }
}
