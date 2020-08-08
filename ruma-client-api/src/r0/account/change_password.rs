//! [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-password)

use ruma_api::ruma_api;

use crate::r0::uiaa::{AuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Change the password of the current user's account.",
        method: POST,
        name: "change_password",
        path: "/_matrix/client/r0/account/password",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The new password for the account.
        pub new_password: String,

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
        pub auth: Option<AuthData>,
    }

    response: {}

    error: UiaaResponse
}
