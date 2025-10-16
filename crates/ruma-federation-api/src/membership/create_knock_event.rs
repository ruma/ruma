//! `PUT /_matrix/federation/*/send_knock/{roomId}/{eventId}`
//!
//! Submits a signed knock event to the resident homeserver for it to accept into the room's graph.

pub mod unstable;
pub mod v1;
