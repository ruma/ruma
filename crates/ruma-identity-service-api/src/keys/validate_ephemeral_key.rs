//! `GET /_matrix/identity/*/pubkey/ephemeral/isvalid`
//!
//! Check whether a short-term public key is valid.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#get_matrixidentityv2pubkeyephemeralisvalid

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
            1.0 => "/_matrix/identity/v2/pubkey/ephemeral/isvalid",
        }
    };

    /// Request type for the `validate_ephemeral_key` endpoint.
    #[request]
    pub struct Request {
        /// The unpadded base64-encoded short-term public key to check.
        #[ruma_api(query)]
        pub public_key: IdentityServerBase64PublicKey,
    }

    /// Response type for the `validate_ephemeral_key` endpoint.
    #[response]
    pub struct Response {
        /// Whether the short-term public key is recognised and is currently valid.
        pub valid: bool,
    }

    impl Request {
        /// Create a `Request` with the given base64-encoded (unpadded) short-term public key.
        pub fn new(public_key: IdentityServerBase64PublicKey) -> Self {
            Self { public_key }
        }
    }

    impl Response {
        /// Create a `Response` with the given bool indicating the validity of the short-term public
        /// key.
        pub fn new(valid: bool) -> Self {
            Self { valid }
        }
    }
}
