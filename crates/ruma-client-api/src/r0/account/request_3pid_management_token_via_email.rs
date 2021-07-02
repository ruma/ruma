//! [POST /_matrix/client/r0/account/3pid/email/requestToken](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-email-requesttoken)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::{ClientSecret, SessionIdBox};

use super::{IdentityServerInfo, IncomingIdentityServerInfo};

ruma_api! {
    metadata: {
        description: "Request a 3PID management token with a 3rd party email.",
        method: POST,
        name: "request_3pid_association_token_via_email",
        path: "/_matrix/client/r0/account/3pid/email/requestToken",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// Client-generated secret string used to protect this session.
        pub client_secret: &'a ClientSecret,

        /// The email address.
        pub email: &'a str,

        /// Used to distinguish protocol level retries from requests to re-send the email.
        pub send_attempt: UInt,

        /// Return URL for identity server to redirect the client back to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_link: Option<&'a str>,

        /// Optional identity server hostname and access token. Deprecated since r0.6.0.
        #[serde(flatten)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub identity_server_info: Option<IdentityServerInfo<'a>>,
    }

    response: {
        /// The session identifier given by the identity server.
        pub sid: SessionIdBox,

        /// URL to submit validation token to. If omitted, verification happens without client.
        ///
        /// If you activate the `compat` feature, this field being an empty string in JSON will give
        /// you `None` here.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
            serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
        )]
        pub submit_url: Option<String>
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the client secret, email and send-attempt counter.
    pub fn new(client_secret: &'a ClientSecret, email: &'a str, send_attempt: UInt) -> Self {
        Self { client_secret, email, send_attempt, next_link: None, identity_server_info: None }
    }
}

impl Response {
    /// Creates a new `Response` with the given session identifier.
    pub fn new(sid: SessionIdBox) -> Self {
        Self { sid, submit_url: None }
    }
}
