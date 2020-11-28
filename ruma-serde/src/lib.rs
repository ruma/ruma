//! De-/serialization helpers for other ruma crates

pub mod can_be_empty;
mod canonical_json;
mod cow;
pub mod duration;
pub mod empty;
pub mod json_string;
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
pub use strings::{
    btreemap_int_or_string_to_int_values, empty_string_as_none, int_or_string_to_int,
};

/// Check whether a value is equal to its default value.
pub fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    val == &T::default()
}

/// Simply returns `true`.
///
/// Useful for `#[serde(default = ...)]`.
pub fn default_true() -> bool {
    true
}

/// Simplfy dereferences the given bool.
///
/// Useful for `#[serde(skip_serializing_if = ...)]`
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_true(b: &bool) -> bool {
    *b
}
