//! `GET /_matrix/key/*/server`
//!
//! Get the homeserver's published signing keys.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixkeyv2server

    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
        serde::Raw,
    };

    use crate::discovery::ServerSigningKeys;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "/_matrix/key/v2/server",
    }

    /// Request type for the `get_server_keys` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_server_keys` endpoint.
    #[response]
    pub struct Response {
        /// Queried server key, signed by the notary server.
        #[ruma_api(body)]
        pub server_key: Raw<ServerSigningKeys>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given server key.
        pub fn new(server_key: Raw<ServerSigningKeys>) -> Self {
            Self { server_key }
        }
    }

    impl From<Raw<ServerSigningKeys>> for Response {
        fn from(server_key: Raw<ServerSigningKeys>) -> Self {
            Self::new(server_key)
        }
    }
}
