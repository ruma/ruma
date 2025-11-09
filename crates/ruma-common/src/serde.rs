//! (De)serialization helpers for other Ruma crates.
//!
//! Part of that is a fork of [serde_urlencoded], with support for sequences in `Deserialize` /
//! `Serialize` structs (e.g. `Vec<Something>`) that are (de)serialized as `field=val1&field=val2`.
//!
//! [serde_urlencoded]: https://github.com/nox/serde_urlencoded

use std::{fmt, marker::PhantomData};

use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeOwned, SeqAccess, Visitor},
};
use serde_json::{Value as JsonValue, value::RawValue as RawJsonValue};
use tracing::debug;

pub mod base64;
mod buf;
pub mod can_be_empty;
mod cow;
pub mod duration;
pub mod json_string;
mod raw;
pub mod single_element_seq;
mod strings;
pub mod test;

pub use self::{
    base64::{Base64, Base64DecodeError},
    buf::{json_to_buf, slice_to_buf},
    can_be_empty::{CanBeEmpty, is_empty},
    cow::deserialize_cow_str,
    raw::{JsonCastable, Raw},
    strings::{
        btreemap_deserialize_v1_powerlevel_values, deserialize_as_number_or_string,
        deserialize_as_optional_number_or_string, deserialize_v1_powerlevel, empty_string_as_none,
        none_as_empty_string,
    },
};

/// The inner type of [`JsonValue::Object`].
pub type JsonObject = serde_json::Map<String, JsonValue>;

/// Check whether a value is equal to its default value.
pub fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    *val == T::default()
}

/// Deserialize a `T` via `Option<T>`, falling back to `T::default()`.
pub fn none_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
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

/// Helper function for returning a default value if deserialization of the type fails.
///
/// Assumes that the content being deserialized is JSON.
///
/// Used as `#[serde(deserialize_with = "default_on_error")]`.
pub fn default_on_error<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = match Box::<RawJsonValue>::deserialize(deserializer) {
        Ok(value) => value,
        Err(error) => {
            debug!("deserialization error, using default value: {error}");
            return Ok(T::default());
        }
    };

    Ok(from_raw_json_value(&value).unwrap_or_else(|error: D::Error| {
        debug!("deserialization error, using default value: {error}");
        T::default()
    }))
}

/// Helper function for ignoring invalid items in a `Vec`, instead letting them cause the entire
/// `Vec` to fail deserialization
pub fn ignore_invalid_vec_items<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct SkipInvalid<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for SkipInvalid<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("Vec with possibly invalid items")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();

            while let Some(result) = seq.next_element::<T>().transpose() {
                let Ok(elem) = result else {
                    continue;
                };

                vec.push(elem);
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(SkipInvalid(PhantomData))
}

pub use ruma_macros::{
    _FakeDeriveSerde, AsRefStr, AsStrAsRefStr, DebugAsRefStr, DeserializeFromCowStr,
    DisplayAsRefStr, EqAsRefStr, FromString, OrdAsRefStr, SerializeAsRefStr, StringEnum,
};
