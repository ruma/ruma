//! `POST /_matrix/identity/*/sign-ed25519`
//!
//! Endpoint to sign invitation details.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#post_matrixidentityv2sign-ed25519

    use ruma_common::{api::ruma_api, serde::Base64, OwnedUserId, ServerSignatures, UserId};

    ruma_api! {
        metadata: {
            description: "Sign invitation details.",
            method: POST,
            name: "sign_invitation_ed25519",
            stable_path: "/_matrix/identity/v2/sign-ed25519",
            authentication: AccessToken,
            rate_limited: false,
            added: 1.0,
        }

        request: {
            /// The Matrix user ID of the user accepting the invitation.
            pub mxid: &'a UserId,

            /// The token from the call to store-invite.
            pub token: &'a str,

            /// The private key, encoded as unpadded base64.
            pub private_key: &'a Base64,
        }

        response: {
            /// The Matrix user ID of the user accepting the invitation.
            pub mxid: OwnedUserId,

            /// The Matrix user ID of the user who sent the invitation.
            pub sender: OwnedUserId,

            /// The signature of the mxid, sender and token.
            pub signatures: ServerSignatures,

            /// The token for the invitation.
            pub token: String,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a `Request` with the given Matrix user ID, token and private_key.
        pub fn new(mxid: &'a UserId, token: &'a str, private_key: &'a Base64) -> Self {
            Self { mxid, token, private_key }
        }
    }

    impl Response {
        /// Creates a `Response` with the given Matrix user ID, sender user ID, signatures and
        /// token.
        pub fn new(
            mxid: OwnedUserId,
            sender: OwnedUserId,
            signatures: ServerSignatures,
            token: String,
        ) -> Self {
            Self { mxid, sender, signatures, token }
        }
    }
}
