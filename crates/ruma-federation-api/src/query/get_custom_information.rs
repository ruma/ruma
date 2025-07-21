//! `GET /_matrix/federation/*/query/{queryType}`
//!
//! Performs a single query request on the receiving homeserver. The query arguments are dependent
//! on which type of query is being made.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1queryquerytype

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };
    use serde_json::Value as JsonValue;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/federation/v1/query/{query_type}",
        }
    };

    /// Request type for the `get_custom_information` endpoint.
    #[request]
    pub struct Request {
        /// The type of query to make.
        #[ruma_api(path)]
        pub query_type: String,

        /// The query parameters.
        #[ruma_api(query_all)]
        pub params: BTreeMap<String, String>,
    }

    /// Response type for the `get_custom_information` endpoint.
    #[response]
    pub struct Response {
        /// The body of the response.
        #[ruma_api(body)]
        pub body: JsonValue,
    }

    impl Request {
        /// Creates a new request with the given type and query parameters.
        pub fn new(query_type: String, params: BTreeMap<String, String>) -> Self {
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
