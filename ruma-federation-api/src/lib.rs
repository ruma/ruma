//! (De)serializable types for the Matrix Federation API.

#![warn(missing_docs)]

mod serde;

pub mod authorization;
pub mod backfill;
pub mod device;
pub mod directory;
pub mod discovery;
pub mod event;
pub mod keys;
pub mod membership;
pub mod openid;
pub mod query;
pub mod thirdparty;
pub mod transactions;
