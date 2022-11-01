//! `GET /_matrix/client/*/account/3pid`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3account3pid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::ThirdPartyIdentifier,
    };

    const METADATA: Metadata = metadata! {
        description: "Get a list of 3rd party contacts associated with the user's account.",
        method: GET,
        name: "get_3pids",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/3pid",
            1.1 => "/_matrix/client/v3/account/3pid",
        }
    };

    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of third party identifiers the homeserver has associated with the user's
        /// account.
        #[serde(default)]
        #[cfg_attr(not(feature = "compat"), serde(skip_serializing_if = "Vec::is_empty"))]
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
