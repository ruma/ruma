//! Endpoints for managing room state

pub mod get_state_event_for_key;
pub mod get_state_events;
#[cfg(feature = "unstable-msc4560")]
pub mod get_stripped_state;
pub mod send_state_event;
