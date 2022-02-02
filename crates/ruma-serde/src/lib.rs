#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serialization helpers for other ruma crates.

#![warn(missing_docs)]

use serde::{de, Deserialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

pub mod base64;
mod buf;
pub mod can_be_empty;
mod canonical_json;
mod cow;
pub mod duration;
mod empty;
pub mod json_string;
mod raw;
pub mod single_element_seq;
mod strings;
pub mod test;
pub mod urlencoded;

pub use self::{
    base64::Base64,
    buf::{json_to_buf, slice_to_buf},
    can_be_empty::{is_empty, CanBeEmpty},
    canonical_json::{
        to_canonical_value, try_from_json_map,
        value::{CanonicalJsonValue, Object as CanonicalJsonObject},
        Error as CanonicalJsonError,
    },
    cow::deserialize_cow_str,
    empty::vec_as_map_of_empty,
    raw::Raw,
    strings::{
        btreemap_deserialize_v1_powerlevel_values, deserialize_v1_powerlevel, empty_string_as_none,
        none_as_empty_string,
    },
};

/// The inner type of [`JsonValue::Object`].
pub type JsonObject = serde_json::Map<String, JsonValue>;

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
/// Useful for `#[serde(skip_serializing_if = ...)]`.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_true(b: &bool) -> bool {
    *b
}

/// Helper function for `serde_json::value::RawValue` deserialization.
pub fn from_raw_json_value<'a, T, E>(val: &'a RawJsonValue) -> Result<T, E>
where
    T: Deserialize<'a>,
    E: de::Error,
{
    serde_json::from_str(val.get()).map_err(E::custom)
}

/// A type that can be sent to another party that understands the matrix protocol.
///
/// If any of the fields of `Self` don't implement serde's `Deserialize`, you can derive this trait
/// to generate a corresponding 'Incoming' type that supports deserialization. This is useful for
/// things like ruma_events' `EventResult` type. For more details, see the
/// [derive macro's documentation][doc].
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
