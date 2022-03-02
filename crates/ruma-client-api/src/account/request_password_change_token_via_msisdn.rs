//! `POST /_matrix/client/*/account/password/msisdn/requestToken`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3accountpasswordmsisdnrequesttoken

    use js_int::UInt;
    use ruma_common::api::ruma_api;
    use ruma_identifiers::{ClientSecret, SessionId};

    ruma_api! {
        metadata: {
            description: "Request that a password change token is sent to the given phone number.",
            method: POST,
            name: "request_password_change_token_via_msisdn",
            r0_path: "/_matrix/client/r0/account/password/msisdn/requestToken",
            stable_path: "/_matrix/client/v3/account/password/msisdn/requestToken",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// Client-generated secret string used to protect this session.
            pub client_secret: &'a ClientSecret,

            /// Two-letter ISO 3166 country code for the phone number.
            pub country: &'a str,

            /// Phone number to validate.
            pub phone_number: &'a str,

            /// Used to distinguish protocol level retries from requests to re-send the SMS.
            pub send_attempt: UInt,

            /// Return URL for identity server to redirect the client back to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub next_link: Option<&'a str>,
        }

        response: {
            /// The session identifier given by the identity server.
            pub sid: Box<SessionId>,

            /// URL to submit validation token to.
            ///
            /// If omitted, verification happens without client.
            ///
            /// If you activate the `compat` feature, this field being an empty string in JSON will result
            /// in `None` here during deserialization.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[cfg_attr(
                feature = "compat",
                serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
            )]
            pub submit_url: Option<String>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given client secret, country code, phone number and
        /// send-attempt counter.
        pub fn new(
            client_secret: &'a ClientSecret,
            country: &'a str,
            phone_number: &'a str,
            send_attempt: UInt,
        ) -> Self {
            Self { client_secret, country, phone_number, send_attempt, next_link: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given session identifier.
        pub fn new(sid: Box<SessionId>) -> Self {
            Self { sid, submit_url: None }
        }
    }
}
