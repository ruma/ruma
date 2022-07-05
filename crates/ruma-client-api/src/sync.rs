//! Endpoints for getting and synchronizing events.

#[cfg(feature = "unstable-msc3575")]
pub mod sliding_sync_events;
pub mod sync_events;
