//! [POST /_matrix/client/r0/account/password/msisdn/requestToken](https://matrix.org/docs/spec/client_server/r0.6.0.html#post-matrix-client-r0-account-password-msisdn-requesttoken)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Request that a password change token is sent to the given phone number.",
        method: POST,
        name: "request_password_change_token_via_msisdn",
        path: "/_matrix/client/r0/account/password/msisdn/requestToken",
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
