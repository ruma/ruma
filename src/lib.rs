//! Crate ruma_events contains serializable types for the events in the [Matrix](https://matrix.org)
//! specification that can be shared by client and server code.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

pub mod call;
pub mod core;
pub mod presence;
pub mod receipt;
pub mod room;
pub mod tag;
pub mod typing;
