//! `GET /_matrix/app/*/thirdparty/protocol/{protocol}`
//!
//! Fetches metadata about the various third party networks that an application service supports.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#get_matrixappv1thirdpartyprotocolprotocol

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
            1.0 => "/_matrix/app/v1/thirdparty/protocol/:protocol",
        }
    };

    /// Request type for the `get_protocol` endpoint.
    #[request]
    pub struct Request {
        /// The name of the protocol.
        #[ruma_api(path)]
        pub protocol: String,
    }

    /// Response type for the `get_protocol` endpoint.
    #[response]
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
