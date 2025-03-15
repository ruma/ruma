//! `GET /_matrix/identity/*/pubkey/isvalid`
//!
//! Check whether a long-term public key is valid. The response should always be the same, provided
//! the key exists.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#get_matrixidentityv2pubkeyisvalid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        third_party_invite::IdentityServerBase64PublicKey,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/identity/v2/pubkey/isvalid",
        }
    };

    /// Request type for the `check_public_key_validity` endpoint.
    #[request]
    pub struct Request {
        /// Base64-encoded (no padding) public key to check for validity.
        #[ruma_api(query)]
        pub public_key: IdentityServerBase64PublicKey,
    }

    /// Response type for the `check_public_key_validity` endpoint.
    #[response]
    pub struct Response {
        /// Whether the public key is recognised and is currently valid.
        pub valid: bool,
    }

    impl Request {
        /// Create a `Request` with the given base64-encoded (unpadded) public key.
        pub fn new(public_key: IdentityServerBase64PublicKey) -> Self {
            Self { public_key }
        }
    }

    impl Response {
        /// Create a `Response` with the given bool indicating the validity of the public key.
        pub fn new(valid: bool) -> Self {
            Self { valid }
        }
    }
}
