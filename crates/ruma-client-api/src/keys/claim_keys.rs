//! `POST /_matrix/client/*/keys/claim`
//!
//! Claims one-time keys for use in pre-key messages.

#[cfg(feature = "unstable-msc3983")]
pub mod msc3983;
pub mod v3;
