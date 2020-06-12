//! Modules for events in the *m.key.verification* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub mod accept;
pub mod cancel;
pub mod key;
pub mod mac;
pub mod request;
pub mod start;

/// A hash algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HashAlgorithm {
    /// The SHA256 hash algorithm.
    Sha256,
}

/// A key agreement protocol.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum KeyAgreementProtocol {
    /// The [Curve25519](https://cr.yp.to/ecdh.html) key agreement protocol.
    Curve25519,
}

/// A message authentication code algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum MessageAuthenticationCode {
    /// The HKDF-HMAC-SHA256 MAC.
    HkdfHmacSha256,
}

/// A Short Authentication String method.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ShortAuthenticationString {
    /// The decimal method.
    Decimal,

    /// The emoji method.
    Emoji,
}

/// A Short Authentication String (SAS) verification method.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[non_exhaustive]
pub enum VerificationMethod {
    /// The *m.sas.v1* verification method.
    #[serde(rename = "m.sas.v1")]
    #[strum(serialize = "m.sas.v1")]
    MSasV1,
}
