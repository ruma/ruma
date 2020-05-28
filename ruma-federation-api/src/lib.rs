//! (De)serializable types for the Matrix Federation API.

#![warn(missing_docs)]

mod serde;

pub mod directory;
pub mod discovery;
pub mod membership;
pub mod pdu;
pub mod query;
pub mod transactions;
