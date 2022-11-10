//! `PUT /_matrix/federation/*/send_leave/{roomId}/{eventId}`
//!
//! Submit a signed leave event to the receiving server for it to accept it into the room's graph.

pub mod v1;
pub mod v2;
