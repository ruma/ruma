//! `PUT /_matrix/federation/*/send_join/{roomId}/{eventId}`
//!
//! Send a join event to a resident server.

#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
pub mod v1;
pub mod v2;
