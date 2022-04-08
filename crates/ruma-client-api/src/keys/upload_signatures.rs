//! `POST /_matrix/client/*/keys/signatures/upload`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3keyssignaturesupload

    use std::collections::BTreeMap;

    use ruma_common::{
        api::ruma_api,
        encryption::{CrossSigningKey, DeviceKeys},
        serde::{Raw, StringEnum},
        OwnedDeviceId, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    use crate::PrivOwnedStr;

    pub use super::iter::SignedKeysIter;

    ruma_api! {
        metadata: {
            description: "Publishes cross-signing signatures for the user.",
            method: POST,
            name: "upload_signatures",
            unstable_path: "/_matrix/client/unstable/keys/signatures/upload",
            stable_path: "/_matrix/client/v3/keys/signatures/upload",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.1,
        }

        request: {
            /// Signed keys.
            #[ruma_api(body)]
            pub signed_keys: BTreeMap<OwnedUserId, SignedKeys>,
        }

        #[derive(Default)]
        response: {
            /// Signature processing failures.
            pub failures: BTreeMap<OwnedUserId, BTreeMap<String, Failure>>,
        }

        error: crate::Error
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
