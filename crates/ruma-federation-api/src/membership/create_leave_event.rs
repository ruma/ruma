//! `PUT /_matrix/federation/*/send_leave/{roomId}/{eventId}`
//!
//! Submit a signed leave event to the receiving server for it to accept it into the room's graph.

#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
pub mod v1;
pub mod v2;
