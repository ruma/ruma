//! Endpoints for the r0.x.x versions of the client API specification.

pub mod account;
pub mod alias;
pub mod appservice;
pub mod backup;
pub mod capabilities;
pub mod config;
pub mod contact;
pub mod context;
pub mod device;
pub mod directory;
pub mod filter;
pub mod keys;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod knock;
pub mod media;
pub mod membership;
pub mod message;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod notification_attribute_data;
pub mod presence;
pub mod profile;
pub mod push;
pub mod read_marker;
pub mod receipt;
pub mod redact;
pub mod room;
pub mod search;
pub mod server;
pub mod session;
pub mod state;
pub mod sync;
pub mod tag;
pub mod thirdparty;
pub mod to_device;
pub mod typing;
pub mod uiaa;
pub mod user_directory;
pub mod voip;
