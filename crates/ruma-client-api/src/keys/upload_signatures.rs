//! `POST /_matrix/client/*/keys/signatures/upload`
//!
//! Publishes cross-signing signatures for the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3keyssignaturesupload

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        encryption::{CrossSigningKey, DeviceKeys},
        metadata,
        serde::{Raw, StringEnum},
        OwnedDeviceId, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    use crate::PrivOwnedStr;

    pub use super::iter::SignedKeysIter;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/keys/signatures/upload",
            1.1 => "/_matrix/client/v3/keys/signatures/upload",
        }
    };

    /// Request type for the `upload_signatures` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Signed keys.
        #[ruma_api(body)]
        pub signed_keys: BTreeMap<OwnedUserId, SignedKeys>,
    }

    /// Response type for the `upload_signatures` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// Signature processing failures.
        pub failures: BTreeMap<OwnedUserId, BTreeMap<String, Failure>>,
    }

    impl Request {
        /// Creates a new `Request` with the given signed keys.
        pub fn new(signed_keys: BTreeMap<OwnedUserId, SignedKeys>) -> Self {
            Self { signed_keys }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }
    }

    /// A map of key IDs to signed key objects.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct SignedKeys(BTreeMap<Box<str>, Box<RawJsonValue>>);

    impl SignedKeys {
        /// Creates an empty `SignedKeys` map.
        pub fn new() -> Self {
            Self::default()
        }

        /// Add the given device keys.
        pub fn add_device_keys(&mut self, device_id: OwnedDeviceId, device_keys: Raw<DeviceKeys>) {
            self.0.insert(device_id.as_str().into(), device_keys.into_json());
        }

        /// Add the given cross signing keys.
        pub fn add_cross_signing_keys(
            &mut self,
            cross_signing_key_id: Box<str>,
            cross_signing_keys: Raw<CrossSigningKey>,
        ) {
            self.0.insert(cross_signing_key_id, cross_signing_keys.into_json());
        }

        /// Returns an iterator over the keys.
        pub fn iter(&self) -> SignedKeysIter<'_> {
            SignedKeysIter(self.0.iter())
        }
    }

    /// A failure to process a signed key.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Failure {
        /// Machine-readable error code.
        errcode: FailureErrorCode,

        /// Human-readable error message.
        error: String,
    }

    /// Error code for signed key processing failures.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
    #[non_exhaustive]
    #[ruma_enum(rename_all = "M_MATRIX_ERROR_CASE")]
    pub enum FailureErrorCode {
        /// The signature is invalid.
        InvalidSignature,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}

mod iter;
