//! `POST /_matrix/identity/*/validate/email/submitToken`
//!
//! Validate an email ID after creation of a session.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#post_matrixidentityv2validateemailsubmittoken

    use ruma_common::api::ruma_api;
    use ruma_identifiers::{ClientSecret, SessionId};

    ruma_api! {
        metadata: {
            description: "Validate ownership of an email address.",
            method: POST,
            name: "validate_email",
            stable_path: "/_matrix/identity/v2/validate/email/submitToken",
            authentication: AccessToken,
            rate_limited: false,
            added: 1.0,
        }

        request: {
            /// The session ID, generated by the `requestToken` call.
            pub sid: &'a SessionId,

            /// The client secret that was supplied to the `requestToken` call.
            pub client_secret: &'a ClientSecret,

            /// The token generated by the `requestToken` call and emailed to the user.
            pub token: &'a str,
        }

        response: {
            /// Whether the validation was successful or not.
            pub success: bool,
        }
    }

    impl<'a> Request<'a> {
        /// Create a new `Request` with the given session ID, client secret and token.
        pub fn new(sid: &'a SessionId, client_secret: &'a ClientSecret, token: &'a str) -> Self {
            Self { sid, client_secret, token }
        }
    }

    impl Response {
        /// Create a new `Response` with the success status.
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}
