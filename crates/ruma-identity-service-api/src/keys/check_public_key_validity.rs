//! `GET /_matrix/identity/*/pubkey/isvalid`
//!
//! Endpoint to check for valid public key with an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#get_matrixidentityv2pubkeyisvalid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Base64,
    };

    const METADATA: Metadata = metadata! {
        description: "Check whether a long-term public key is valid. The response should always be the same, provided the key exists.",
        method: GET,
        name: "check_public_key_validity",
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/identity/v2/pubkey/isvalid",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// Base64-encoded (no padding) public key to check for validity.
        #[ruma_api(query)]
        pub public_key: &'a Base64,
    }

    #[response]
    pub struct Response {
        /// Whether the public key is recognised and is currently valid.
        pub valid: bool,
    }

    impl<'a> Request<'a> {
        /// Create a `Request` with the given base64-encoded (unpadded) public key.
        pub fn new(public_key: &'a Base64) -> Self {
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
