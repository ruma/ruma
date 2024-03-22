//! Server discovery endpoints.

pub mod discover_homeserver;
pub mod discover_support;
#[cfg(feature = "unstable-msc2965")]
pub mod get_authentication_issuer;
pub mod get_capabilities;
pub mod get_supported_versions;
