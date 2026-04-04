//! Public and private key pairs.

use std::collections::BTreeMap;

use ruma_common::serde::Base64;

use crate::signatures::Signature;

/// A cryptographic key pair for digitally signing data.
pub trait KeyPair: Sized {
    /// Signs a JSON object.
    ///
    /// # Parameters
    ///
    /// * `message`: An arbitrary series of bytes to sign.
    fn sign(&self, message: &[u8]) -> Signature;
}

/// A map from entity names to sets of public keys for that entity.
///
/// An entity is generally a homeserver, e.g. `example.com`.
pub type PublicKeyMap = BTreeMap<String, PublicKeySet>;

/// A set of public keys for a single homeserver.
///
/// This is represented as a map from key ID to base64-encoded signature.
pub type PublicKeySet = BTreeMap<String, Base64>;
