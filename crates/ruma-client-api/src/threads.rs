//! Endpoints for querying threads in a room.

#[cfg(feature = "unstable-msc4306")]
pub mod get_thread_subscription;
pub mod get_threads;
#[cfg(feature = "unstable-msc4306")]
pub mod subscribe_thread;
#[cfg(feature = "unstable-msc4306")]
pub mod unsubscribe_thread;
