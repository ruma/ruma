//! `GET /_matrix/client/*/register/m.login.registration_token/validity`
//!
//! Checks to see if the given registration token is valid.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1registermloginregistration_tokenvalidity

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3231/register/org.matrix.msc3231.login.registration_token/validity",
            1.2 => "/_matrix/client/v1/register/m.login.registration_token/validity",
        }
    };

    /// Request type for the `check_registration_token_validity` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The registration token to check the validity of.
        #[ruma_api(query)]
        pub token: String,
    }

    /// Response type for the `check_registration_token_validity` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A flag to indicate that the registration token is valid.
        pub valid: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given registration token.
        pub fn new(token: String) -> Self {
            Self { token }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given validity flag.
        pub fn new(valid: bool) -> Self {
            Self { valid }
        }
    }
}
