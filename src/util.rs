use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{EventJson, TryFromRaw};

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

// This would be #[cfg(test)] if it wasn't used from external tests
pub fn serde_json_eq_try_from_raw<T>(de: T, se: serde_json::Value)
where
    T: Clone + Debug + PartialEq + Serialize + TryFromRaw,
{
    assert_eq!(se, serde_json::to_value(de.clone()).unwrap());
    assert_eq!(
        de,
        serde_json::from_value::<EventJson<_>>(se)
            .unwrap()
            .deserialize()
            .unwrap()
    );
}
