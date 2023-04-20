//! `POST /_matrix/client/*/account/password/msisdn/requestToken`
//!
//! Request that a password change token is sent to the given phone number.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3accountpasswordmsisdnrequesttoken

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedClientSecret, OwnedSessionId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/account/password/msisdn/requestToken",
            1.1 => "/_matrix/client/v3/account/password/msisdn/requestToken",
        }
    };

    /// Request type for the `request_password_change_token_via_msisdn` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Client-generated secret string used to protect this session.
        pub client_secret: OwnedClientSecret,

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

    /// Response type for the `request_password_change_token_via_msisdn` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The session identifier given by the identity server.
        pub sid: OwnedSessionId,

        /// URL to submit validation token to.
        ///
        /// If omitted, verification happens without client.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub submit_url: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given client secret, country code, phone number and
        /// send-attempt counter.
        pub fn new(
            client_secret: OwnedClientSecret,
            country: String,
            phone_number: String,
            send_attempt: UInt,
        ) -> Self {
            Self { client_secret, country, phone_number, send_attempt, next_link: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given session identifier.
        pub fn new(sid: OwnedSessionId) -> Self {
            Self { sid, submit_url: None }
        }
    }
}
