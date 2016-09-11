//! `x-www-form-urlencoded` meets Serde

#[macro_use]
extern crate serde;
extern crate url;

pub mod de;
pub mod ser;

pub use de::{Deserializer, from_bytes, from_str};
pub use ser::{Serializer, to_string};
