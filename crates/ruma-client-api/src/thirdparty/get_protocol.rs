//! `GET /_matrix/client/*/thirdparty/protocol/{protocol}`
//!
//! Fetches the metadata from the homeserver about a particular third party protocol.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3thirdpartyprotocolprotocol

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
            1.0 => "/_matrix/client/r0/thirdparty/protocol/{protocol}",
            1.1 => "/_matrix/client/v3/thirdparty/protocol/{protocol}",
        }
    };

    /// Request type for the `get_protocol` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The name of the protocol.
        #[ruma_api(path)]
        pub protocol: String,
    }

    /// Response type for the `get_protocol` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Metadata about the protocol.
        #[ruma_api(body)]
        pub protocol: Protocol,
    }

    impl Request {
        /// Creates a new `Request` with the given protocol name.
        pub fn new(protocol: String) -> Self {
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
