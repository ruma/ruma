//! `GET /_matrix/client/*/register/m.login.registration_token/validity`

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv1registermloginregistration_tokenvalidity

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Checks to see if the given registration token is valid.",
        method: GET,
        name: "check_registration_token_validity",
        rate_limited: true,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc3231/register/org.matrix.msc3231.login.registration_token/validity",
            1.2 => "/_matrix/client/v1/register/m.login.registration_token/validity",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The registration token to check the validity of.
        #[ruma_api(query)]
        pub registration_token: &'a str,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// A flag to indicate that the registration token is valid.
        pub valid: bool,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given registration token.
        pub fn new(registration_token: &'a str) -> Self {
            Self { registration_token }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given validity flag.
        pub fn new(valid: bool) -> Self {
            Self { valid }
        }
    }
}
