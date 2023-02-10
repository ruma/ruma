//! `POST /_matrix/identity/*/3pid/bind`
//!
//! Publish an association between a session and a Matrix user ID.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#post_matrixidentityv23pidbind

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Medium,
        MilliSecondsSinceUnixEpoch, OwnedClientSecret, OwnedSessionId, OwnedUserId,
        ServerSignatures,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/3pid/bind",
        }
    };

    /// Request type for the `bind_3pid` endpoint.
    #[request]
    pub struct Request {
        /// The session ID generated by the `requestToken` call.
        pub sid: OwnedSessionId,

        /// The client secret passed to the `requestToken` call.
        pub client_secret: OwnedClientSecret,

        /// The Matrix user ID to associate with the 3PIDs.
        pub mxid: OwnedUserId,
    }

    /// Response type for the `bind_3pid` endpoint.
    #[response]
    pub struct Response {
        /// The 3PID address of the user being looked up.
        pub address: String,

        /// The medium type of the 3PID.
        pub medium: Medium,

        /// The Matrix user ID associated with the 3PID.
        pub mxid: OwnedUserId,

        /// A UNIX timestamp before which the association is not known to be valid.
        pub not_before: MilliSecondsSinceUnixEpoch,

        /// A UNIX timestamp after which the association is not known to be valid.
        pub not_after: MilliSecondsSinceUnixEpoch,

        /// The UNIX timestamp at which the association was verified.
        pub ts: MilliSecondsSinceUnixEpoch,

        /// The signatures of the verifiying identity servers which show that the
        /// association should be trusted, if you trust the verifying identity services.
        pub signatures: ServerSignatures,
    }

    impl Request {
        /// Creates a `Request` with the given session ID, client secret and Matrix user ID.
        pub fn new(
            sid: OwnedSessionId,
            client_secret: OwnedClientSecret,
            mxid: OwnedUserId,
        ) -> Self {
            Self { sid, client_secret, mxid }
        }
    }

    impl Response {
        /// Creates a `Response` with the given 3PID address, medium, Matrix user ID, timestamps and
        /// signatures.
        pub fn new(
            address: String,
            medium: Medium,
            mxid: OwnedUserId,
            not_before: MilliSecondsSinceUnixEpoch,
            not_after: MilliSecondsSinceUnixEpoch,
            ts: MilliSecondsSinceUnixEpoch,
            signatures: ServerSignatures,
        ) -> Self {
            Self { address, medium, mxid, not_before, not_after, ts, signatures }
        }
    }
}
