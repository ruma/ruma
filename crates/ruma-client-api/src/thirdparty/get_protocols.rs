//! `GET /_matrix/client/*/thirdparty/protocols`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3thirdpartyprotocols

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, thirdparty::Protocol};

    ruma_api! {
        metadata: {
            description: "Fetches the overall metadata about protocols supported by the homeserver.",
            method: GET,
            name: "get_protocols",
            r0_path: "/_matrix/client/r0/thirdparty/protocols",
            stable_path: "/_matrix/client/v3/thirdparty/protocols",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// Metadata about protocols supported by the homeserver.
            #[ruma_api(body)]
            pub protocols: BTreeMap<String, Protocol>,
        }

        error: crate::Error
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
