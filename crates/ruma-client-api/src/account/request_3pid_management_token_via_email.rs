//! `POST /_matrix/client/*/account/3pid/email/requestToken`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3account3pidemailrequesttoken

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, ClientSecret, OwnedSessionId,
    };

    use crate::account::{IdentityServerInfo, IncomingIdentityServerInfo};

    const METADATA: Metadata = metadata! {
        description: "Request a 3PID management token with a 3rd party email.",
        method: POST,
        name: "request_3pid_management_token_via_email",
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/account/3pid/email/requestToken",
            1.1 => "/_matrix/client/v3/account/3pid/email/requestToken",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// Client-generated secret string used to protect this session.
        pub client_secret: &'a ClientSecret,

        /// The email address.
        pub email: &'a str,

        /// Used to distinguish protocol level retries from requests to re-send the email.
        pub send_attempt: UInt,

        /// Return URL for identity server to redirect the client back to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_link: Option<&'a str>,

        /// Optional identity server hostname and access token.
        ///
        /// Deprecated since r0.6.0.
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pub identity_server_info: Option<IdentityServerInfo<'a>>,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// The session identifier given by the identity server.
        pub sid: OwnedSessionId,

        /// URL to submit validation token to.
        ///
        /// If omitted, verification happens without client.
        ///
        /// If you activate the `compat` feature, this field being an empty string in JSON will
        /// result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub submit_url: Option<String>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the client secret, email and send-attempt counter.
        pub fn new(client_secret: &'a ClientSecret, email: &'a str, send_attempt: UInt) -> Self {
            Self { client_secret, email, send_attempt, next_link: None, identity_server_info: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given session identifier.
        pub fn new(sid: OwnedSessionId) -> Self {
            Self { sid, submit_url: None }
        }
    }
}
