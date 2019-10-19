use crate::{EventType, TryFromRaw};

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

pub fn get_type_field<E: serde::de::Error>(value: &serde_json::Value) -> Result<EventType, E> {
    serde_json::from_value(
        value
            .get("type")
            .cloned()
            .ok_or_else(|| E::missing_field("type"))?,
    )
    .map_err(serde_json_error_to_generic_de_error)
}
