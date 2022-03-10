//! `GET /_matrix/key/*/server`
//!
//! Endpoint for retrieving a server's published signing keys.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! Note: The specification includes `/{keyID}`, but this is deprecated, and the trailing slash
    //! then made optional.
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixkeyv2serverkeyid

    use ruma_common::{api::ruma_api, serde::Raw};

    use crate::discovery::ServerSigningKeys;

    ruma_api! {
        metadata: {
            description: "Gets the homeserver's published signing keys.",
            method: GET,
            name: "get_server_keys",
            stable_path: "/_matrix/key/v2/server",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// Queried server key, signed by the notary server.
            #[ruma_api(body)]
            pub server_key: Raw<ServerSigningKeys>,
        }
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
