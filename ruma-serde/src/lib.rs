//! De-/serialization helpers for other ruma crates

use js_int::Int;
use serde::{
    de::{Error, IntoDeserializer},
    Deserialize,
};
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

pub mod can_be_empty;
mod canonical_json;
mod cow;
pub mod duration;
pub mod empty;
pub mod json_string;
pub mod single_element_seq;
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

/// Serde deserialization decorator to map empty Strings to None,
/// and forward non-empty Strings to the Deserialize implementation for T.
/// Useful for the typical
/// "A room with an X event with an absent, null, or empty Y field
/// should be treated the same as a room with no such event."
/// formulation in the spec.
///
/// To be used like this:
/// `#[serde(deserialize_with = "empty_string_as_none")]`
/// Relevant serde issue: <https://github.com/serde-rs/serde/issues/1425>
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        // If T = String, like in m.room.name, the second deserialize is actually superfluous.
        // TODO: optimize that somehow?
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

// Helper type for deserialize_int_or_string_to_int
#[derive(Deserialize)]
#[serde(untagged)]
enum IntOrString<'a> {
    Num(Int),
    Str(&'a str),
}

impl TryFrom<IntOrString<'_>> for Int {
    type Error = js_int::ParseIntError;

    fn try_from(input: IntOrString) -> Result<Self, Self::Error> {
        match input {
            IntOrString::Num(n) => Ok(n),
            IntOrString::Str(string) => string.parse(),
        }
    }
}

/// Take either an integer number or a string and deserialize to an integer number.
///
/// To be used like this:
/// `#[serde(deserialize_with = "int_or_string_to_int")]`
pub fn int_or_string_to_int<'de, D>(de: D) -> Result<Int, D::Error>
where
    D: serde::Deserializer<'de>,
{
    IntOrString::deserialize(de)?.try_into().map_err(D::Error::custom)
}

/// Take a BTreeMap with values of either an integer number or a string and deserialize
/// those to integer numbers.
///
/// To be used like this:
/// `#[serde(deserialize_with = "btreemap_int_or_string_to_int_values")]`
pub fn btreemap_int_or_string_to_int_values<'de, D, T>(de: D) -> Result<BTreeMap<T, Int>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Ord,
{
    BTreeMap::<T, IntOrString>::deserialize(de)?
        .into_iter()
        .map(|(k, v)| v.try_into().map(|n| (k, n)).map_err(D::Error::custom))
        .collect()
}
