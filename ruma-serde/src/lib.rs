//! De-/serialization helpers for other ruma crates

pub mod can_be_empty;
mod canonical_json;
mod cow;
pub mod duration;
pub mod empty;
pub mod json_string;
mod raw;
pub mod single_element_seq;
mod strings;
pub mod test;
pub mod time;
pub mod urlencoded;

pub use can_be_empty::{is_empty, CanBeEmpty};
pub use canonical_json::{
    to_canonical_value, to_string as to_canonical_json_string, try_from_json_map,
    value::{CanonicalJsonValue, Object as CanonicalJsonObject},
    Error as CanonicalJsonError,
};
pub use cow::deserialize_cow_str;
pub use empty::vec_as_map_of_empty;
pub use raw::Raw;
pub use strings::{
    btreemap_int_or_string_to_int_values, empty_string_as_none, int_or_string_to_int,
};

/// Check whether a value is equal to its default value.
pub fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    *val == T::default()
}

/// Simply returns `true`.
///
/// Useful for `#[serde(default = ...)]`.
pub fn default_true() -> bool {
    true
}

/// Simply dereferences the given bool.
///
/// Useful for `#[serde(skip_serializing_if = ...)]`
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_true(b: &bool) -> bool {
    *b
}

/// A type that can be sent to another party that understands the matrix protocol. If any of the
/// fields of `Self` don't implement serde's `Deserialize`, you can derive this trait to generate a
/// corresponding 'Incoming' type that supports deserialization. This is useful for things like
/// ruma_events' `EventResult` type. For more details, see the [derive macro's documentation][doc].
///
/// [doc]: derive.Outgoing.html
// TODO: Better explain how this trait relates to serde's traits
pub trait Outgoing {
    /// The 'Incoming' variant of `Self`.
    type Incoming;
}

// -- Everything below is macro-related --

pub use ruma_serde_macros::*;

/// This module is used to support the generated code from ruma-serde-macros.
/// It is not considered part of ruma-serde's public API.
#[doc(hidden)]
pub mod exports {
    pub use serde;
}
