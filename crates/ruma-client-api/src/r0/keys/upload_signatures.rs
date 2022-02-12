//! [POST /_matrix/client/r0/keys/signatures/upload](https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3keyssignaturesupload)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::encryption::{CrossSigningKey, DeviceKeys};
use ruma_identifiers::{DeviceId, UserId};
use ruma_serde::{Raw, StringEnum};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::PrivOwnedStr;

ruma_api! {
    metadata: {
        description: "Publishes cross-signing signatures for the user.",
        method: POST,
        name: "upload_signatures",
        path: "/_matrix/client/r0/keys/signatures/upload",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// Signed keys.
        #[ruma_api(body)]
        pub signed_keys: BTreeMap<Box<UserId>, SignedKeys>,
    }

    #[derive(Default)]
    response: {
        /// Signature processing failures.
        pub failures: BTreeMap<Box<UserId>, BTreeMap<String, Failure>>,
    }

    error: crate::Error
}

impl Request {
    /// Creates a new `Request` with the given signed keys.
    pub fn new(signed_keys: BTreeMap<Box<UserId>, SignedKeys>) -> Self {
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
    pub fn add_device_keys(&mut self, device_id: Box<DeviceId>, device_keys: Raw<DeviceKeys>) {
        self.0.insert(device_id.into(), device_keys.into_json());
    }

    /// Add the given cross signing keys.
    pub fn add_cross_signing_keys(
        &mut self,
        what_the_hell_is_this: Box<str>,
        cross_signing_keys: Raw<CrossSigningKey>,
    ) {
        self.0.insert(what_the_hell_is_this, cross_signing_keys.into_json());
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
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
#[ruma_enum(rename_all = "M_MATRIX_ERROR_CASE")]
pub enum FailureErrorCode {
    /// The signature is invalid.
    InvalidSignature,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
