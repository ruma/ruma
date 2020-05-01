//! De-/serialization helpers for other ruma crates

use serde::de::{Deserialize, IntoDeserializer};

pub mod duration;
pub mod empty;
pub mod json_string;
pub mod test;
pub mod time;
pub mod urlencoded;

pub use empty::vec_as_map_of_empty;

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
pub fn is_true(b: &bool) -> bool {
    *b
}

/// Serde deserialization decorator to map empty Strings to None,
/// and forward non-empty Strings to the Deserialize implementation for T.
/// Useful for the typical
/// "A room with an X event with an absent, null, or empty Y field
/// should be treated the same as a room with no such event."
/// formulation in the spec.
///
/// To be used like this:
/// `#[serde(deserialize_with = "empty_string_as_none"]`
/// Relevant serde issue: https://github.com/serde-rs/serde/issues/1425
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    // TODO: Switch to and remove this attribute `opt.as_deref()` once MSRV is >= 1.40
    #[allow(clippy::option_as_ref_deref, clippy::unknown_clippy_lints)]
    let opt = opt.as_ref().map(String::as_str);
    match opt {
        None | Some("") => Ok(None),
        // If T = String, like in m.room.name, the second deserialize is actually superfluous.
        // TODO: optimize that somehow?
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}
