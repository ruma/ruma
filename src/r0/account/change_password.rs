//! [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-password)

use ruma_api::ruma_api;

use super::AuthenticationData;

ruma_api! {
    metadata {
        description: "Change the password of the current user's account.",
        method: POST,
        name: "change_password",
        path: "/_matrix/client/r0/account/password",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The new password for the account.
        pub new_password: String,
        /// Additional authentication information for the user-interactive authentication API.
        pub auth: Option<AuthenticationData>,
    }

    response {}
}
