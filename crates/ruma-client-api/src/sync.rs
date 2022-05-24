//! Endpoints for getting and synchronizing events.

pub mod sync_events;
#[cfg(feature = "unstable-msc3575")]
pub mod sliding_sync_events;
