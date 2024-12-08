//! `GET /_matrix/federation/*/version`
//!
//! Get the implementation name and version of this homeserver.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1version

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/federation/v1/version",
        }
    };

    /// Request type for the `get_server_version` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_server_version` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// Information about the homeserver implementation
        #[serde(skip_serializing_if = "Option::is_none")]
        pub server: Option<Server>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// Arbitrary values that identify this implementation.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Server {
        /// Arbitrary name that identifies this implementation.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// Version of this implementation.
        ///
        /// The version format depends on the implementation.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub version: Option<String>,
    }

    impl Server {
        /// Creates an empty `Server`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
