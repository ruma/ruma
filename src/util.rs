use serde::de::{Deserialize, DeserializeOwned, IntoDeserializer};

use crate::TryFromRaw;

pub fn try_convert_variant<Enum: TryFromRaw, Content: TryFromRaw>(
    raw_variant: fn(Content::Raw) -> Enum::Raw,
    variant: fn(Content) -> Enum,
    raw: Content::Raw,
) -> Result<Enum, (String, Enum::Raw)> {
    Content::try_from_raw(raw)
        .map(variant)
        .map_err(|(err, raw)| (err.to_string(), raw_variant(raw)))
}

pub fn serde_json_error_to_generic_de_error<E: serde::de::Error>(error: serde_json::Error) -> E {
    E::custom(error.to_string())
}

pub fn get_field<T: DeserializeOwned, E: serde::de::Error>(
    value: &serde_json::Value,
    field: &'static str,
) -> Result<T, E> {
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
