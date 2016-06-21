//! Crate ruma_common contains serializable types for the events and APIs in the
//! [Matrix](https://matrix.org) client-server specification that can be shared by client and
//! server code.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;

pub mod events;
