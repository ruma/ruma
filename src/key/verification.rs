//! Modules for events in the *m.key.verification* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use serde::{Deserialize, Serialize};

pub mod accept;
pub mod cancel;
pub mod key;
pub mod mac;
pub mod request;
pub mod start;

/// A hash algorithm.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Deserialize)]
pub enum HashAlgorithm {
    /// The SHA256 hash algorithm.
    #[serde(rename = "sha256")]
    Sha256,
}

impl_enum! {
    HashAlgorithm {
        Sha256 => "sha256",
    }
}

/// A key agreement protocol.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Deserialize)]
pub enum KeyAgreementProtocol {
    /// The [Curve25519](https://cr.yp.to/ecdh.html) key agreement protocol.
    #[serde(rename = "curve25519")]
    Curve25519,
}

impl_enum! {
    KeyAgreementProtocol {
        Curve25519 => "curve25519",
    }
}

/// A message authentication code algorithm.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Deserialize)]
pub enum MessageAuthenticationCode {
    /// The HKDF-HMAC-SHA256 MAC.
    #[serde(rename = "hkdf-hmac-sha256")]
    HkdfHmacSha256,
}

impl_enum! {
    MessageAuthenticationCode {
        HkdfHmacSha256 => "hkdf-hmac-sha256",
    }
}

/// A Short Authentication String method.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Deserialize)]
pub enum ShortAuthenticationString {
    /// The decimal method.
    #[serde(rename = "decimal")]
    Decimal,

    /// The emoji method.
    #[serde(rename = "emoji")]
    Emoji,
}

impl_enum! {
    ShortAuthenticationString {
        Decimal => "decimal",
        Emoji => "emoji",
    }
}

/// A Short Authentication String (SAS) verification method.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum VerificationMethod {
    /// The *m.sas.v1* verification method.
    #[serde(rename = "m.sas.v1")]
    MSasV1,
}

impl_enum! {
    VerificationMethod {
        MSasV1 => "m.sas.v1",
    }
}
