//! `GET /_matrix/federation/*/hierarchy/{roomId}`
//!
//! Get the space tree in a depth-first manner to locate child rooms of a given space.

pub mod unstable;
pub mod v1;
