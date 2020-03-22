//! [POST /_matrix/client/r0/account/3pid/msisdn/requestToken](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-msisdn-requesttoken)

use js_int::UInt;
use ruma_api::ruma_api;

use super::IdentityServerInfo;

ruma_api! {
    metadata {
        description: "Request a 3PID management token with a phone number.",
        method: POST,
        name: "request_3pid_association_token_via_msisdn",
        path: "/_matrix/client/r0/account/3pid/msisdn/requestToken",
        rate_limited: false,
        requires_authentication: false,
    }

    request {
        /// Client-generated secret string used to protect this session.
        pub client_secret: String,
        /// Two-letter ISO 3166 country code for the phone number.
        pub country: String,
        /// Phone number to validate.
        pub phone_number: String,
        /// Used to distinguish protocol level retries from requests to re-send the SMS.
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
