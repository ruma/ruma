//! [GET /_matrix/client/r0/login/sso/redirect](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-login-sso-redirect)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "",
        method: GET,
        name: "sso_login",
        path: "/_matrix/client/r0/login/sso/redirect",
        rate_limited: false,
        authentication: None,

    }

    request: {
        /// URL to which the homeserver should return the user after completing
        /// authentication with the SSO identity provider.
        #[ruma_api(query)]
        pub redirect_url: &'a str,
    }

    response: {
        /// Redirect URL to the SSO identity provider.
        #[ruma_api(header = LOCATION)]
        pub location: String,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given redirect URL.
    pub fn new(redirect_url: &'a str) -> Self {
        Self { redirect_url }
    }
}

impl Response {
    /// Creates a new `Response` with the given SSO URL.
    pub fn new(location: String) -> Self {
        Self { location }
    }
}
