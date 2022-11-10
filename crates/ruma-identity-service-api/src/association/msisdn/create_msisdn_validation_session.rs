//! `POST /_matrix/identity/*/validate/msisdn/requestToken`
//!
//! Create a session for validation of a phone number.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#post_matrixidentityv2validatemsisdnrequesttoken

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, ClientSecret, OwnedSessionId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/validate/msisdn/requestToken",
        }
    };

    /// Request type for the `create_msisdn_validation_session` endpoint.
    #[request]
    pub struct Request<'a> {
        /// A unique string generated by the client, and used to identify the validation attempt.
        pub client_secret: &'a ClientSecret,

        /// The two-letter uppercase ISO-3166-1 alpha-2 country code that the number in
        /// `phone_number` should be parsed as if it were dialled from.
        pub country: &'a str,

        /// The phone number to validate.
        pub phone_number: &'a str,

        /// The server will only send an SMS if the send_attempt is a number greater than the most
        /// recent one which it has seen, scoped to that `country` + `phone_number` +
        /// `client_secret` triple.
        pub send_attempt: UInt,

        /// When the validation is completed, the identity server will redirect the user to this
        /// URL.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_link: Option<&'a str>,
    }

    /// Response type for the `create_msisdn_validation_session` endpoint.
    #[response]
    pub struct Response {
        /// The session ID.
        ///
        /// Session IDs are opaque strings generated by the identity server.
        pub sid: OwnedSessionId,
    }

    impl<'a> Request<'a> {
        /// Create a new `Request` with the given client secret, country code, phone number, the
        /// `send_attempt` number and the next link to go to after validation.
        pub fn new(
            client_secret: &'a ClientSecret,
            country: &'a str,
            phone_number: &'a str,
            send_attempt: UInt,
            next_link: Option<&'a str>,
        ) -> Self {
            Self { client_secret, country, phone_number, send_attempt, next_link }
        }
    }

    impl Response {
        /// Create a new `Response` with the given session ID.
        pub fn new(sid: OwnedSessionId) -> Self {
            Self { sid }
        }
    }
}
