//! [GET /_matrix/client/r0/login/sso/redirect](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-login-sso-redirect)

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "",
        method: GET,
        name: "sso_login",
        path: "/_matrix/client/r0/login/sso/redirect",
        rate_limited: false,
        requires_authentication: false,

    }

    request {
        /// URL to which the homeserver should return the user after completing
        /// authentication with the SSO identity provider.
        #[ruma_api(query)]
        pub redirect_url: String,
    }

    response {
        /// Redirect URL to the SSO identity provider.
        #[ruma_api(header = LOCATION)]
        pub location: String,
    }

    error: crate::Error
}
