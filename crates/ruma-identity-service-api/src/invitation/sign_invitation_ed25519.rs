//! `POST /_matrix/identity/*/sign-ed25519`
//!
//! Sign invitation details.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#post_matrixidentityv2sign-ed25519

    use ruma_common::{
        ServerSignatures, UserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::Base64,
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/sign-ed25519",
        }
    }

    /// Request type for the `sign_invitation_ed25519` endpoint.
    #[request]
    pub struct Request {
        /// The Matrix user ID of the user accepting the invitation.
        pub mxid: UserId,

        /// The token from the call to store-invite.
        pub token: String,

        /// The private key, encoded as unpadded base64.
        pub private_key: Base64,
    }

    /// Response type for the `sign_invitation_ed25519` endpoint.
    #[response]
    pub struct Response {
        /// The Matrix user ID of the user accepting the invitation.
        pub mxid: UserId,

        /// The Matrix user ID of the user who sent the invitation.
        pub sender: UserId,

        /// The signature of the mxid, sender and token.
        pub signatures: ServerSignatures,

        /// The token for the invitation.
        pub token: String,
    }

    impl Request {
        /// Creates a `Request` with the given Matrix user ID, token and private_key.
        pub fn new(mxid: UserId, token: String, private_key: Base64) -> Self {
            Self { mxid, token, private_key }
        }
    }

    impl Response {
        /// Creates a `Response` with the given Matrix user ID, sender user ID, signatures and
        /// token.
        pub fn new(
            mxid: UserId,
            sender: UserId,
            signatures: ServerSignatures,
            token: String,
        ) -> Self {
            Self { mxid, sender, signatures, token }
        }
    }
}
