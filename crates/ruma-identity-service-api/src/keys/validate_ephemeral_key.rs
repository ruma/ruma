//! `GET /_matrix/identity/*/pubkey/ephemeral/isvalid`
//!
//! Endpoint to check for validity of short-term public key.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#get_matrixidentityv2pubkeyephemeralisvalid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Base64,
    };

    const METADATA: Metadata = metadata! {
        description: "Check whether a short-term public key is valid.",
        method: GET,
        name: "validate_ephemeral_key",
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/identity/v2/pubkey/ephemeral/isvalid",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The unpadded base64-encoded short-term public key to check.
        #[ruma_api(query)]
        pub public_key: &'a Base64,
    }

    #[response]
    pub struct Response {
        /// Whether the short-term public key is recognised and is currently valid.
        pub valid: bool,
    }

    impl<'a> Request<'a> {
        /// Create a `Request` with the given base64-encoded (unpadded) short-term public key.
        pub fn new(public_key: &'a Base64) -> Self {
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
