//! [GET /_matrix/client/unstable/org.matrix.msc3231/register/org.matrix.msc3231.login.registration_token/validity](https://github.com/matrix-org/matrix-doc/blob/main/proposals/3231-token-authenticated-registration.md#checking-the-validity-of-a-token)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Checks to see if the given registration token is valid.",
        method: GET,
        name: "check_registration_token_validity",
        path: "/_matrix/client/unstable/org.matrix.msc3231/register/org.matrix.msc3231.login.registration_token/validity",
        rate_limited: true,
        authentication: None,
    }

    request: {
        /// The registration token to check the validity of.
        #[ruma_api(query)]
        pub registration_token: &'a str,
    }

    response: {
        /// A flag to indicate that the registration token is valid.
        pub valid: bool,
    }

    error: crate::Error
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
