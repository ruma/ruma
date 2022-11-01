//! `GET /_matrix/client/*/thirdparty/protocols`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3thirdpartyprotocols

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Protocol,
    };

    const METADATA: Metadata = metadata! {
        description: "Fetches the overall metadata about protocols supported by the homeserver.",
        method: GET,
        name: "get_protocols",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/thirdparty/protocols",
            1.1 => "/_matrix/client/v3/thirdparty/protocols",
        }
    };

    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

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
