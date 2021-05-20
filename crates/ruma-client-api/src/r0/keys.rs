//! Endpoints for key management

pub use ruma_common::encryption::{
    CrossSigningKey, CrossSigningKeySignatures, KeyUsage, OneTimeKey, SignedKey,
    SignedKeySignatures,
};

pub mod claim_keys;
pub mod get_key_changes;
pub mod get_keys;
pub mod upload_keys;

#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod upload_signatures;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod upload_signing_keys;
