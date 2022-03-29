//! `GET /_matrix/identity/*/pubkey/isvalid`
//!
//! Endpoint to check for valid public key with an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityv2pubkeyisvalid

    use ruma_common::{api::ruma_api, serde::Base64};

    ruma_api! {
        metadata: {
            description: "Check whether a long-term public key is valid. The response should always be the same, provided the key exists.",
            method: GET,
            name: "check_public_key_validity",
            stable_path: "/_matrix/identity/v2/pubkey/isvalid",
            authentication: None,
            rate_limited: false,
            added: 1.0,
        }

        request: {
            /// Base64-encoded (no padding) public key to check for validity.
            #[ruma_api(query)]
            pub public_key: &'a Base64,
        }

        response: {
            /// Whether the public key is recognised and is currently valid.
            pub valid: bool,
        }
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
