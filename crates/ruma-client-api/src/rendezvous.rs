//! Endpoints for managing rendezvous sessions.

pub mod create_rendezvous_session;
#[cfg(feature = "unstable-msc4388")]
pub mod delete_rendezvous_session;
#[cfg(feature = "unstable-msc4388")]
pub mod get_rendezvous_session;
#[cfg(feature = "unstable-msc4388")]
pub mod update_rendezvous_session;
