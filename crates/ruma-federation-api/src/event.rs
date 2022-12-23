//! Endpoints to get general information about events

pub mod get_event;
#[cfg(feature = "unstable-msc3030")]
pub mod get_event_by_timestamp;
pub mod get_missing_events;
pub mod get_room_state;
pub mod get_room_state_ids;
