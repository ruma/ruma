//! [POST /_matrix/client/r0/register/email/requestToken](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-register-email-requesttoken)

use js_int::UInt;
use ruma_api::ruma_api;

use super::IdentityServerInfo;

ruma_api! {
    metadata {
        description: "Request a registration token with a 3rd party email.",
        method: POST,
        name: "request_registration_token_via_email",
        path: "/_matrix/client/r0/register/email/requestToken",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// Client-generated secret string used to protect this session.
        pub client_secret: String,
        /// The email address.
        pub email: String,
        /// Used to distinguish protocol level retries from requests to re-send the email.
        pub send_attempt: UInt,
        /// Return URL for identity server to redirect the client back to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_link: Option<String>,
        /// Optional identity server hostname and access token. Deprecated since r0.6.0.
        #[serde(flatten)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub identity_server_info: Option<IdentityServerInfo>,
    }

    response {
        /// The session identifier given by the identity server.
        pub sid: String,
        /// URL to submit validation token to. If omitted, verification happens without client.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub submit_url: Option<String>
    }

    error: crate::Error
}
