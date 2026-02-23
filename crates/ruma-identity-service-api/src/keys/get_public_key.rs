//! `GET /_matrix/identity/*/pubkey/{keyId}`
//!
//! Get the public key for the given key ID.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#get_matrixidentityv2pubkeykeyid

    use ruma_common::{
        ServerSigningKeyId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
        third_party_invite::IdentityServerBase64PublicKey,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/identity/v2/pubkey/{key_id}",
        }
    }

    /// Request type for the `get_public_key` endpoint.
    #[request]
    pub struct Request {
        /// The ID of the key.
        #[ruma_api(path)]
        pub key_id: ServerSigningKeyId,
    }

    /// Response type for the `get_public_key` endpoint.
    #[response]
    pub struct Response {
        /// Unpadded base64-encoded public key.
        pub public_key: IdentityServerBase64PublicKey,
    }

    impl Request {
        /// Create a `Request` with the given key_id.
        pub fn new(key_id: ServerSigningKeyId) -> Self {
            Self { key_id }
        }
    }

    impl Response {
        /// Create a `Response` with the given base64-encoded (unpadded) public key.
        pub fn new(public_key: IdentityServerBase64PublicKey) -> Self {
            Self { public_key }
        }
    }
}
