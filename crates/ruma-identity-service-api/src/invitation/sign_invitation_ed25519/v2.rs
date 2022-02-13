//![POST /_matrix/identity/v2/sign-ed25519](https://matrix.org/docs/spec/identity_service/r0.3.0#post-matrix-identity-v2-sign-ed25519)

use ruma_api::ruma_api;
use ruma_identifiers::{ServerSignatures, UserId};
use ruma_serde::Base64;

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
        pub mxid: Box<UserId>,

        /// The Matrix user ID of the user who sent the invitation.
        pub sender: Box<UserId>,

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
    /// Creates a `Response` with the given Matrix user ID, sender user ID, signatures and token.
    pub fn new(
        mxid: Box<UserId>,
        sender: Box<UserId>,
        signatures: ServerSignatures,
        token: String,
    ) -> Self {
        Self { mxid, sender, signatures, token }
    }
}
