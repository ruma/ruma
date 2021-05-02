//! [POST /_matrix/client/r0/account/3pid/email/requestToken](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-email-requesttoken)

use js_int::UInt;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Ask for a verification token for a given 3rd party ID.",
        method: POST,
        name: "request_contact_verification_token",
        path: "/_matrix/client/r0/account/3pid/email/requestToken",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// Client-generated secret string used to protect this session.
        pub client_secret: &'a str,

        /// The email address.
        pub email: &'a str,

        /// Used to distinguish protocol level retries from requests to re-send
        /// the email.
        pub send_attempt: UInt,

        /// A URL for the identity server to redirect the user to after
        /// validation is completed.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_link: Option<&'a str>,

        /// The identity server to send the onward request to as a hostname with
        /// an appended colon and port number if the port is not the default.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<&'a str>,

        /// An access token previously registered with the identity server.
        ///
        /// Required if an `id_server` is supplied.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_access_token: Option<&'a str>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given client secret, email and send-attempt counter.
    pub fn new(client_secret: &'a str, email: &'a str, send_attempt: UInt) -> Self {
        Self {
            client_secret,
            email,
            send_attempt,
            next_link: None,
            id_server: None,
            id_access_token: None,
        }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
