//! Endpoints to get general information about events

pub mod get_event;
pub mod get_event_by_timestamp;
pub mod get_missing_events;
pub mod get_room_state;
pub mod get_room_state_ids;
#[cfg(feature = "unstable-msc4560")]
pub mod get_stripped_state;
