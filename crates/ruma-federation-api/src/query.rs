//! Endpoints to retrieve information from a homeserver about a resource.

pub mod get_custom_information;
#[cfg(feature = "unstable-msc4495")]
pub mod get_presence_recipients;
pub mod get_profile_information;
pub mod get_room_information;
