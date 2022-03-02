//! `GET /_matrix/client/*/thirdparty/protocol/{protocol}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3thirdpartyprotocolprotocol

    use ruma_common::{api::ruma_api, thirdparty::Protocol};

    ruma_api! {
        metadata: {
            description: "Fetches the metadata from the homeserver about a particular third party protocol.",
            method: GET,
            name: "get_protocol",
            r0_path: "/_matrix/client/r0/thirdparty/protocol/:protocol",
            stable_path: "/_matrix/client/v3/thirdparty/protocol/:protocol",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The name of the protocol.
            #[ruma_api(path)]
            pub protocol: &'a str,
        }

        response: {
            /// Metadata about the protocol.
            #[ruma_api(body)]
            pub protocol: Protocol,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given protocol name.
        pub fn new(protocol: &'a str) -> Self {
            Self { protocol }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given protocol.
        pub fn new(protocol: Protocol) -> Self {
            Self { protocol }
        }
    }
}
