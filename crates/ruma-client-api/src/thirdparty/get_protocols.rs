//! `GET /_matrix/client/*/thirdparty/protocols`
//!
//! Fetches the overall metadata about protocols supported by the homeserver.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3thirdpartyprotocols

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Protocol,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/thirdparty/protocols",
            1.1 => "/_matrix/client/v3/thirdparty/protocols",
        }
    };

    /// Request type for the `get_protocols` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_protocols` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Metadata about protocols supported by the homeserver.
        #[ruma_api(body)]
        pub protocols: BTreeMap<String, Protocol>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given protocols.
        pub fn new(protocols: BTreeMap<String, Protocol>) -> Self {
            Self { protocols }
        }
    }
}
