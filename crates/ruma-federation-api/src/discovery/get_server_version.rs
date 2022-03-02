//! `GET /_matrix/federation/*/version`
//!
//! Endpoint to retrieve metadata about a server implementation.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1version

    use ruma_common::api::ruma_api;
    use serde::{Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Get the implementation name and version of this homeserver.",
            method: GET,
            name: "get_server_version",
            stable_path: "/_matrix/federation/v1/version",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        #[derive(Default)]
        response: {
            /// Information about the homeserver implementation
            #[serde(skip_serializing_if = "Option::is_none")]
            pub server: Option<Server>,
        }
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
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
