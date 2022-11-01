//! `GET /_matrix/identity/*/pubkey/{keyId}`
//!
//! Endpoint to retrieve the public key for a key ID.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#get_matrixidentityv2pubkeykeyid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Base64,
        ServerSigningKeyId,
    };

    const METADATA: Metadata = metadata! {
        description: "Get the public key for the given key ID.",
        method: GET,
        name: "get_public_key",
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/identity/v2/pubkey/:key_id",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The ID of the key.
        #[ruma_api(path)]
        pub key_id: &'a ServerSigningKeyId,
    }

    #[response]
    pub struct Response {
        /// Unpadded base64-encoded public key.
        pub public_key: Base64,
    }

    impl<'a> Request<'a> {
        /// Create a `Request` with the given key_id.
        pub fn new(key_id: &'a ServerSigningKeyId) -> Self {
            Self { key_id }
        }
    }

    impl Response {
        /// Create a `Response` with the given base64-encoded (unpadded) public key.
        pub fn new(public_key: Base64) -> Self {
            Self { public_key }
        }
    }
}
