use serde::de::{Deserialize, DeserializeOwned, IntoDeserializer};
use serde_json::Value;

use crate::TryFromRaw;

pub fn try_convert_variant<Enum: TryFromRaw, Content: TryFromRaw>(
    variant: fn(Content) -> Enum,
    raw: Content::Raw,
) -> Result<Enum, String> {
    Content::try_from_raw(raw)
        .map(variant)
        .map_err(|err| err.to_string())
}

pub fn try_variant_from_value<T, U, E>(value: Value, variant: fn(T) -> U) -> Result<U, E>
where
    T: DeserializeOwned,
    E: serde::de::Error,
{
    serde_json::from_value(value)
        .map(variant)
        .map_err(serde_json_error_to_generic_de_error)
}

pub fn serde_json_error_to_generic_de_error<E: serde::de::Error>(error: serde_json::Error) -> E {
    E::custom(error.to_string())
}

pub fn get_field<T, E>(value: &Value, field: &'static str) -> Result<T, E>
where
    T: DeserializeOwned,
    E: serde::de::Error,
{
    serde_json::from_value(
        value
            .get(field)
            .cloned()
            .ok_or_else(|| E::missing_field(field))?,
    )
    .map_err(serde_json_error_to_generic_de_error)
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

/// Serde serialization and deserialization functions that map a `Vec<T>` to a `HashMap<T, Empty>`.
///
/// The Matrix spec sometimes specifies lists as hash maps so the list entries can be expanded with
/// attributes without breaking compatibility. As that would be a breaking change for ruma's event
/// types anyway, we convert them to `Vec`s for simplicity, using this module.
///
/// To be used as `#[serde(with = "vec_as_map_of_empty")]`.
pub mod vec_as_map_of_empty {
    use crate::Empty;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::{collections::HashMap, hash::Hash};

    #[allow(clippy::ptr_arg)]
    pub fn serialize<S, T>(vec: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + Hash + Eq,
    {
        vec.iter()
            .map(|v| (v, Empty))
            .collect::<HashMap<_, _>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Hash + Eq,
    {
        HashMap::<T, Empty>::deserialize(deserializer)
            .map(|hashmap| hashmap.into_iter().map(|(k, _)| k).collect())
    }
}

/// Used to default the `bool` fields to `true` during deserialization.
pub fn default_true() -> bool {
    true
}

#[cfg(test)]
mod test_util {
    use std::fmt::Debug;

    use serde::{de::DeserializeOwned, Serialize};

    use crate::{EventResult, TryFromRaw};

    pub fn serde_json_eq<T>(de: T, se: serde_json::Value)
    where
        T: Clone + Debug + PartialEq + Serialize + DeserializeOwned,
    {
        assert_eq!(se, serde_json::to_value(de.clone()).unwrap());
        assert_eq!(de, serde_json::from_value(se).unwrap());
    }

    pub fn serde_json_eq_try_from_raw<T>(de: T, se: serde_json::Value)
    where
        T: Clone + Debug + PartialEq + Serialize + TryFromRaw,
    {
        assert_eq!(se, serde_json::to_value(de.clone()).unwrap());
        assert_eq!(
            de,
            serde_json::from_value::<EventResult<_>>(se)
                .unwrap()
                .into_result()
                .unwrap()
        );
    }
}

#[cfg(test)]
pub use test_util::*;
