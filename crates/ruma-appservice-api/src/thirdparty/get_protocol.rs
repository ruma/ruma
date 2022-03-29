//! `GET /_matrix/app/*/thirdparty/protocol/{protocol}`
//!
//! Endpoint to present clients with specific information about the various third party networks
//! that an application service supports.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#get_matrixappv1thirdpartyprotocolprotocol

    use ruma_common::{api::ruma_api, thirdparty::Protocol};

    ruma_api! {
        metadata: {
            description: "Fetches the metadata from the homeserver about a particular third party protocol.",
            method: GET,
            name: "get_protocol",
            stable_path: "/_matrix/app/v1/thirdparty/protocol/:protocol",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
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
