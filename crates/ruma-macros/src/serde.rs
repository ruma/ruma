//! Methods and types for [ruma-serde].
//!
//! [ruma-serde]: https://github.com/ruma/ruma/tree/main/ruma-serde

pub mod attr;
pub mod case;
pub mod deserialize_from_cow_str;
pub mod display_as_ref_str;
pub mod enum_as_ref_str;
pub mod enum_from_string;
pub mod eq_as_ref_str;
pub mod ord_as_ref_str;
pub mod outgoing;
pub mod serialize_as_ref_str;
mod util;
