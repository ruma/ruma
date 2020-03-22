//! [POST /_matrix/client/r0/account/3pid/email/requestToken](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-account-3pid-email-requesttoken)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Ask for a verification token for a given 3rd party ID.",
        method: POST,
        name: "request_contact_verification_token",
        path: "/_matrix/client/r0/account/3pid/email/requestToken",
        rate_limited: false,
        requires_authentication: false,
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
        pub send_attempt: UInt,
    }

    response {}

    error: crate::Error
}
