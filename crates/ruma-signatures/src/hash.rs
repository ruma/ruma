use base64::{Engine, alphabet};
use ruma_common::{
    CanonicalJsonObject, CanonicalJsonValue,
    canonical_json::{CanonicalJsonObjectExt, redact},
    room_version_rules::{EventIdFormatVersion, RoomVersionRules},
    serde::{Base64, base64::Standard},
};
use sha2::{Digest, Sha256};

use crate::{JsonError, verify::to_canonical_json_string_with_fields_to_remove};

/// The [maximum size allowed] for a PDU.
///
/// [maximum size allowed]: https://spec.matrix.org/v1.18/client-server-api/#size-limits
const MAX_PDU_BYTES: usize = 65_535;

/// The fields to remove from a JSON object when creating a content hash of an event.
static CONTENT_HASH_FIELDS_TO_REMOVE: &[&str] = &["hashes", "signatures", "unsigned"];

/// The fields to remove from a JSON object when creating a reference hash of an event.
static REFERENCE_HASH_FIELDS_TO_REMOVE: &[&str] = &["signatures", "unsigned"];

/// Compute and add the [content hash] to the given event.
///
/// This adds or overwrites the `sha256` key in the `hashes` object of the event.
///
/// This should only be called when creating a new event.
///
/// # Parameters
///
/// * `object`: A JSON object to be hashed according to the Matrix specification.
///
/// # Errors
///
/// Returns an error if the `hashes` key is present and is not an object.
///
/// # Examples
///
/// ```
/// use ruma_common::CanonicalJsonObject;
/// use ruma_signatures::hash_event;
///
/// // Deserialize an event from JSON.
/// let mut event = serde_json::from_str(
///     r#"{
///         "room_id": "!x:domain",
///         "sender": "@a:domain",
///         "origin": "domain",
///         "origin_server_ts": 1000000,
///         "type": "X",
///         "content": {},
///         "prev_events": [],
///         "auth_events": [],
///         "depth": 3
///     }"#,
/// )?;
///
/// // Hash the JSON.
/// hash_event(&mut event)?;
///
/// // The hash was added.
/// assert_eq!(
///     event,
///     serde_json::from_str::<CanonicalJsonObject>(
///         r#"{
///             "room_id": "!x:domain",
///             "sender": "@a:domain",
///             "origin": "domain",
///             "origin_server_ts": 1000000,
///             "type": "X",
///             "content": {},
///             "prev_events": [],
///             "auth_events": [],
///             "depth": 3,
///             "hashes": {
///                 "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
///             }
///         }"#,
///     )?
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [content hash]: https://spec.matrix.org/v1.18/server-server-api/#calculating-the-content-hash-for-an-event
pub fn hash_event(object: &mut CanonicalJsonObject) -> Result<(), JsonError> {
    let hash = content_hash(object)?;

    let hashes = object.get_as_object_or_insert_default("hashes", "hashes")?;
    hashes.insert("sha256".into(), CanonicalJsonValue::String(hash.encode()));

    Ok(())
}

/// Computes the [content hash] of the given event.
///
/// The content hash of an event covers the complete event including the unredacted contents. It is
/// used during federation and is described in the Matrix server-server specification.
///
/// # Parameters
///
/// * `object`: A JSON object to generate a content hash for.
///
/// # Errors
///
/// Returns an error if the event is too large.
///
/// [content hash]: https://spec.matrix.org/v1.18/server-server-api/#calculating-the-content-hash-for-an-event
pub fn content_hash(object: &CanonicalJsonObject) -> Result<Base64<Standard, [u8; 32]>, JsonError> {
    let json =
        to_canonical_json_string_with_fields_to_remove(object, CONTENT_HASH_FIELDS_TO_REMOVE)?;

    if json.len() > MAX_PDU_BYTES {
        return Err(JsonError::PduTooLarge);
    }

    let hash = Sha256::digest(json.as_bytes());

    Ok(Base64::new(hash.into()))
}

/// Computes the [reference hash] of the given event.
///
/// The reference hash of an event covers the essential fields of an event, including content
/// hashes.
///
/// When creating a new event, [`hash_event()`] must be called before this function to add the
/// content hash.
///
/// Returns the hash as a base64-encoded string, without padding. The correct character set is used
/// depending on the room version:
///
/// * For room versions 1 and 2, the standard character set is used for sending the reference hash
///   of the `auth_events` and `prev_events`.
/// * For room version 3, the standard character set is used for using the reference hash as the
///   event ID.
/// * For newer versions, the URL-safe character set is used for using the reference hash as the
///   event ID.
///
/// # Parameters
///
/// * `object`: A JSON object to generate a reference hash for.
/// * `rules`: The rules of the version of the current room.
///
/// # Errors
///
/// Returns an error if the event is too large or redaction fails.
///
/// [reference hash]: https://spec.matrix.org/v1.18/server-server-api#calculating-the-reference-hash-for-an-event
pub fn reference_hash(
    object: &CanonicalJsonObject,
    rules: &RoomVersionRules,
) -> Result<String, JsonError> {
    let redacted_value = redact(object.clone(), &rules.redaction, None)?;

    let json = to_canonical_json_string_with_fields_to_remove(
        &redacted_value,
        REFERENCE_HASH_FIELDS_TO_REMOVE,
    )?;

    if json.len() > MAX_PDU_BYTES {
        return Err(JsonError::PduTooLarge);
    }

    let hash = Sha256::digest(json.as_bytes());

    let base64_alphabet = match rules.event_id_format {
        EventIdFormatVersion::V1 | EventIdFormatVersion::V2 => alphabet::STANDARD,
        // Room versions higher than version 3 are URL-safe base64 encoded
        _ => alphabet::URL_SAFE,
    };
    let base64_engine = base64::engine::GeneralPurpose::new(
        &base64_alphabet,
        base64::engine::general_purpose::NO_PAD,
    );

    Ok(base64_engine.encode(hash))
}
