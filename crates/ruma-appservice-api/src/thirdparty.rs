//! Endpoints for third party lookups

pub mod get_location_for_protocol;
pub mod get_location_for_room_alias;
pub mod get_protocol;
#[cfg(feature = "unstable-msc4417")]
pub mod get_url_preview;
pub mod get_user_for_protocol;
pub mod get_user_for_user_id;
