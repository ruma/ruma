//! `GET /_matrix/client/*/account/3pid`
//!
//! Get a list of 3rd party contacts associated with the user's account.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3account3pid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::ThirdPartyIdentifier,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/3pid",
            1.1 => "/_matrix/client/v3/account/3pid",
        }
    };

    /// Request type for the `get_3pids` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_3pids` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of third party identifiers the homeserver has associated with the user's
        /// account.
        ///
        /// If the `compat-get-3pids` feature is enabled, this field will always be serialized,
        /// even if its value is an empty list.
        #[serde(default)]
        #[cfg_attr(
            not(feature = "compat-get-3pids"),
            serde(skip_serializing_if = "Vec::is_empty")
        )]
        pub threepids: Vec<ThirdPartyIdentifier>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given third party identifiers.
        pub fn new(threepids: Vec<ThirdPartyIdentifier>) -> Self {
            Self { threepids }
        }
    }
}
