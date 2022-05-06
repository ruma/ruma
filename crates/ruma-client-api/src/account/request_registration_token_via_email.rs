//! `POST /_matrix/client/*/register/email/requestToken`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3registeremailrequesttoken

    use js_int::UInt;
    use ruma_common::{api::ruma_api, ClientSecret, OwnedSessionId};

    use crate::account::{IdentityServerInfo, IncomingIdentityServerInfo};

    ruma_api! {
        metadata: {
            description: "Request a registration token with a 3rd party email.",
            method: POST,
            name: "request_registration_token_via_email",
            r0_path: "/_matrix/client/r0/register/email/requestToken",
            stable_path: "/_matrix/client/v3/register/email/requestToken",
            rate_limited: false,
            authentication: None,
            added: 1.0,
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

            /// Optional identity server hostname and access token.
            ///
            /// Deprecated since r0.6.0.
            #[serde(flatten, skip_serializing_if = "Option::is_none")]
            pub identity_server_info: Option<IdentityServerInfo<'a>>,
        }

        response: {
            /// The session identifier given by the identity server.
            pub sid: OwnedSessionId,

            /// URL to submit validation token to.
            ///
            /// If omitted, verification happens without client.
            ///
            /// If you activate the `compat` feature, this field being an empty string in JSON will result
            /// in `None` here during deserialization.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[cfg_attr(
                feature = "compat",
                serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
            )]
            pub submit_url: Option<String>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given client secret, email address and send-attempt
        /// counter.
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
