//! [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-password)

use ruma_api::ruma_api;

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Change the password of the current user's account.",
        method: POST,
        name: "change_password",
        path: "/_matrix/client/r0/account/password",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The new password for the account.
        pub new_password: &'a str,

        /// True to revoke the user's other access tokens, and their associated devices if the
        /// request succeeds.
        ///
        /// Defaults to true.
        ///
        /// When false, the server can still take advantage of the soft logout method for the user's
        /// remaining devices.
        #[serde(default = "ruma_serde::default_true", skip_serializing_if = "ruma_serde::is_true")]
        pub logout_devices: bool,

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,
    }

    #[derive(Default)]
    response: {}

    error: UiaaResponse
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given password.
    pub fn new(new_password: &'a str) -> Self {
        Self { new_password, logout_devices: true, auth: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
