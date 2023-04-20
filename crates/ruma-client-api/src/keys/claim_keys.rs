//! `POST /_matrix/client/*/keys/claim`
//!
//! Claims one-time keys for use in pre-key messages.

pub mod v3;
#[cfg(feature = "unstable-msc3983")]
pub mod v4;
