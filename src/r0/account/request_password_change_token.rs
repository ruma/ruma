//! [POST /_matrix/client/r0/account/password/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password-email-requesttoken)

use ruma_api_macros::ruma_api;
use serde_derive::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Request that a password change token is sent to the given email address.",
        method: POST,
        name: "request_password_change_token",
        path: "/_matrix/client/r0/account/password/email/requestToken",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// TODO: This parameter is not documented in the spec.
        pub client_secret: String,
        /// TODO: This parameter is not documented in the spec.
        pub email: String,
        /// TODO: This parameter is not documented in the spec.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,
        /// TODO: This parameter is not documented in the spec.
        pub send_attempt: u64,
    }

    response {}
}
