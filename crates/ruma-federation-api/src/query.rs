//! Endpoints to retrieve information from a homeserver about a resource.

pub mod get_custom_information;
#[cfg(feature = "unstable-msc4373")]
pub mod get_edu_types;
pub mod get_profile_information;
pub mod get_room_information;
