use js_int::int;
use ruma_common::{
    room_version_rules::EventFormatRules, CanonicalJsonObject, CanonicalJsonValue, ID_MAX_BYTES,
};
use serde_json::to_string as to_json_string;

/// The [maximum size allowed] for a PDU.
///
/// [maximum size allowed]: https://spec.matrix.org/latest/client-server-api/#size-limits
const MAX_PDU_BYTES: usize = 65_535;

/// The [maximum length allowed] for the `prev_events` array of a PDU.
///
/// [maximum length allowed]: https://spec.matrix.org/latest/rooms/v1/#event-format
const MAX_PREV_EVENTS_LENGTH: usize = 20;

/// The [maximum length allowed] for the `auth_events` array of a PDU.
///
/// [maximum length allowed]: https://spec.matrix.org/latest/rooms/v1/#event-format
const MAX_AUTH_EVENTS_LENGTH: usize = 10;

/// Check that the given canonicalized PDU respects the event format of the room version and the
/// [size limits] from the Matrix specification.
///
/// This is part of the [checks performed on receipt of a PDU].
///
/// This checks the following and enforces their size limits:
///
/// * Full PDU
/// * `sender`
/// * `room_id`
/// * `type`
/// * `event_id`
/// * `state_key`
/// * `prev_events`
/// * `auth_events`
/// * `depth`
///
/// Returns an `Err(_)` if the JSON is malformed or if the PDU doesn't pass the checks.
///
/// [size limits]: https://spec.matrix.org/latest/client-server-api/#size-limits
/// [checks performed on receipt of a PDU]: https://spec.matrix.org/latest/server-server-api/#checks-performed-on-receipt-of-a-pdu
pub fn check_pdu_format(pdu: &CanonicalJsonObject, rules: &EventFormatRules) -> Result<(), String> {
    // Check the PDU size, it must occur on the full PDU with signatures.
    let json =
        to_json_string(&pdu).map_err(|e| format!("Failed to serialize canonical JSON: {e}"))?;
    if json.len() > MAX_PDU_BYTES {
        return Err("PDU is larger than maximum of {MAX_PDU_BYTES} bytes".to_owned());
    }

    // Check the presence, type and length of the `sender`, `room_id` and `type` fields.
    for field in &["sender", "room_id", "type"] {
        let value = extract_string_field(pdu, field)?
            .ok_or_else(|| format!("missing `{field}` field in PDU"))?;

        if value.len() > ID_MAX_BYTES {
            return Err(format!(
                "invalid `{field}` field in PDU: \
                 string length is larger than maximum of {ID_MAX_BYTES} bytes"
            ));
        }
    }

    // Check the presence, type and length of the `event_id` field.
    let event_id = extract_string_field(pdu, "event_id")?;

    if rules.require_event_id && event_id.is_none() {
        return Err("missing `event_id` field in PDU".to_owned());
    }

    if event_id.is_some_and(|event_id| event_id.len() > ID_MAX_BYTES) {
        return Err(format!(
            "invalid `event_id` field in PDU: \
             string length is larger than maximum of {ID_MAX_BYTES} bytes"
        ));
    }

    // Check the type and length of the `state_key` field.
    if extract_string_field(pdu, "state_key")?
        .is_some_and(|state_key| state_key.len() > ID_MAX_BYTES)
    {
        return Err(format!(
            "invalid `state_key` field in PDU: \
             string length is larger than maximum of {ID_MAX_BYTES} bytes"
        ));
    }

    // Check the presence, type and length of the `auth_events` and `prev_events` fields.
    for (field, max_value) in
        &[("auth_events", MAX_AUTH_EVENTS_LENGTH), ("prev_events", MAX_PREV_EVENTS_LENGTH)]
    {
        let value = extract_array_field(pdu, field)?
            .ok_or_else(|| format!("missing `{field}` field in PDU"))?;

        if value.len() > *max_value {
            return Err(format!(
                "invalid `{field}` field in PDU: \
                 array length is larger than maximum of {max_value}"
            ));
        }
    }

    // Check the presence, type and value of the `depth` field.
    match pdu.get("depth") {
        Some(CanonicalJsonValue::Integer(value)) => {
            if *value < int!(0) {
                return Err("invalid `depth` field in PDU: cannot be a negative integer".to_owned());
            }
        }
        Some(value) => {
            return Err(format!(
                "unexpected format of `depth` field in PDU: \
                 expected integer, got {value:?}"
            ))
        }
        None => return Err("missing `depth` field in PDU".to_owned()),
    }

    Ok(())
}

/// Extract the string field with the given name from the given canonical JSON object.
///
/// Returns `Ok(Some(value))` if the field is present and a string, `Ok(None)` if the fields is
/// missing and an error if the field is present and not a string.
fn extract_string_field<'a>(
    object: &'a CanonicalJsonObject,
    field: &'a str,
) -> Result<Option<&'a String>, String> {
    match object.get(field) {
        Some(CanonicalJsonValue::String(value)) => Ok(Some(value)),
        Some(value) => Err(format!(
            "unexpected format of `{field}` field in PDU: \
             expected string, got {value:?}"
        )),
        None => Ok(None),
    }
}

/// Extract the array field with the given name from the given canonical JSON object.
///
/// Returns `Ok(Some(value))` if the field is present and an array, `Ok(None)` if the fields is
/// missing and an error if the field is present and not a array.
fn extract_array_field<'a>(
    object: &'a CanonicalJsonObject,
    field: &'a str,
) -> Result<Option<&'a [CanonicalJsonValue]>, String> {
    match object.get(field) {
        Some(CanonicalJsonValue::Array(value)) => Ok(Some(value)),
        Some(value) => Err(format!(
            "unexpected format of `{field}` field in PDU: \
             expected array, got {value:?}"
        )),
        None => Ok(None),
    }
}
