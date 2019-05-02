//! [POST /_matrix/client/r0/register/email/requestToken](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-register-email-requesttoken)

use ruma_api_macros::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Request a register token with a 3rd party email.",
        method: POST,
        name: "request_register_token",
        path: "/_matrix/client/r0/register/email/requestToken",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// Client-generated secret string used to protect this session.
        pub client_secret: String,
        /// The email address.
        pub email: String,
        /// The ID server to send the onward request to as a hostname with an appended colon and port number if the port is not the default.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,
        /// Used to distinguish protocol level retries from requests to re-send the email.
        pub send_attempt: u64,
    }

    response {}
}
