//! Functions for signing and verifying JSON and events.

use std::error::Error as _;

use base64::{encode_config, STANDARD_NO_PAD};
use ring::digest::{digest, SHA256};
use serde_json::{json, to_string, to_value, Value};

use crate::{
    keys::KeyPair,
    signatures::{Signature, SignatureMap, SignatureSet},
    verification::Verifier,
    Error,
};

/// The fields that are allowed to remain in an event during redaction.
static ALLOWED_KEYS: &[&str] = &[
    "event_id",
    "type",
    "room_id",
    "sender",
    "state_key",
    "content",
    "hashes",
    "signatures",
    "depth",
    "prev_events",
    "prev_state",
    "auth_events",
    "origin",
    "origin_server_ts",
    "membership",
];

/// The fields of an *m.room.power_levels* event's `content` key that are allowed to remain in an
/// event during redaction.
static ALLOWED_POWER_LEVELS_KEYS: &[&str] = &[
    "ban",
    "events",
    "events_default",
    "kick",
    "redact",
    "state_default",
    "users",
    "users_default",
];

/// The fields to remove from a JSON object when converting JSON into the "canonical" form.
static CANONICAL_JSON_FIELDS_TO_REMOVE: &[&str] = &["signatures", "unsigned"];

/// The fields to remove from a JSON object when creating a content hash of an event.
static CONTENT_HASH_FIELDS_TO_REMOVE: &[&str] = &["hashes", "signatures", "unsigned"];

/// The fields to remove from a JSON object when creating a reference hash of an event.
static REFERENCE_HASH_FIELDS_TO_REMOVE: &[&str] = &["age_ts", "signatures", "unsigned"];

/// Signs an arbitrary JSON object.
///
/// # Parameters
///
/// * key_pair: A cryptographic key pair used to sign the JSON.
/// * value: A JSON object to be signed according to the Matrix specification.
///
/// # Errors
///
/// Returns an error if the JSON value is not a JSON object.
pub fn sign_json<K>(key_pair: &K, value: &Value) -> Result<Signature, Error>
where
    K: KeyPair,
{
    let json = to_canonical_json(value)?;

    Ok(key_pair.sign(json.as_bytes()))
}

/// Converts a JSON object into the "canonical" string form, suitable for signing.
///
/// # Parameters
///
/// * value: The `serde_json::Value` (JSON value) to convert.
///
/// # Errors
///
/// Returns an error if the provided JSON value is not a JSON object.
pub fn to_canonical_json(value: &Value) -> Result<String, Error> {
    to_canonical_json_with_fields_to_remove(value, CANONICAL_JSON_FIELDS_TO_REMOVE)
}

/// Use a public key to verify a signature of a JSON object.
///
/// # Parameters
///
/// * verifier: A `Verifier` appropriate for the digital signature algorithm that was used.
/// * public_key: The public key of the key pair used to sign the JSON, as a series of bytes.
/// * signature: The `Signature` to verify.
/// * value: The `serde_json::Value` (JSON value) that was signed.
///
/// # Errors
///
/// Returns an error if verification fails.
pub fn verify_json<V>(
    verifier: &V,
    public_key: &[u8],
    signature: &Signature,
    value: &Value,
) -> Result<(), Error>
where
    V: Verifier,
{
    verifier.verify_json(public_key, signature, to_canonical_json(value)?.as_bytes())
}

/// Creates a *content hash* for the JSON representation of an event.
///
/// The content hash of an event covers the complete event including the unredacted contents. It is
/// used during federation and is described in the Matrix server-server specification.
pub fn content_hash(value: &Value) -> Result<String, Error> {
    let json = to_canonical_json_with_fields_to_remove(value, CONTENT_HASH_FIELDS_TO_REMOVE)?;

    let hash = digest(&SHA256, json.as_bytes());

    Ok(encode_config(&hash, STANDARD_NO_PAD))
}

/// Creates a *reference hash* for the JSON representation of an event.
///
/// The reference hash of an event covers the essential fields of an event, including content
/// hashes. It is used during federation and is described in the Matrix server-server
/// specification.
pub fn reference_hash(value: &Value) -> Result<String, Error> {
    let redacted_value = redact(value)?;

    let json =
        to_canonical_json_with_fields_to_remove(&redacted_value, REFERENCE_HASH_FIELDS_TO_REMOVE)?;

    let hash = digest(&SHA256, json.as_bytes());

    Ok(encode_config(&hash, STANDARD_NO_PAD))
}

/// Hashes and signs the JSON representation of an event.
///
/// # Parameters
///
/// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
/// * key_pair: A cryptographic key pair used to sign the event.
/// * value: A JSON object to be hashed and signed according to the Matrix specification.
pub fn hash_and_sign_event<K>(
    server_name: &str,
    key_pair: &K,
    value: &Value,
) -> Result<Value, Error>
where
    K: KeyPair,
{
    let hash = content_hash(value)?;

    if !value.is_object() {
        return Err(Error::new("JSON value must be a JSON object"));
    }

    let mut owned_value = value.clone();

    // Limit the scope of the mutable borrow so `owned_value` can be passed immutably to `redact`
    // below.
    {
        let object = owned_value
            .as_object_mut()
            .expect("safe since we checked above");

        let hashes = json!({ "sha256": hash });

        object.insert("hashes".to_string(), hashes);
    }

    let redacted = redact(&owned_value)?;

    let signature = sign_json(key_pair, &redacted)?;

    let mut signature_set = SignatureSet::with_capacity(1);
    signature_set.insert(signature);

    let mut signature_map = SignatureMap::with_capacity(1);
    signature_map.insert(server_name, signature_set)?;

    let signature_map_value =
        to_value(signature_map).map_err(|error| Error::new(error.to_string()))?;

    let object = owned_value
        .as_object_mut()
        .expect("safe since we checked above");
    object.insert("signatures".to_string(), signature_map_value);

    Ok(owned_value)
}

/// Internal implementation detail of the canonical JSON algorithm. Allows customization of the
/// fields that will be removed before serializing.
fn to_canonical_json_with_fields_to_remove(
    value: &Value,
    fields: &[&str],
) -> Result<String, Error> {
    if !value.is_object() {
        return Err(Error::new("JSON value must be a JSON object"));
    }

    let mut owned_value = value.clone();

    {
        let object = owned_value
            .as_object_mut()
            .expect("safe since we checked above");

        for field in fields {
            object.remove(*field);
        }
    }

    to_string(&owned_value).map_err(|error| Error::new(error.description()))
}

/// Redact the JSON representation of an event using the rules specified in the Matrix
/// client-server specification.
///
/// This is part of the process of signing an event.
fn redact(value: &Value) -> Result<Value, Error> {
    if !value.is_object() {
        return Err(Error::new("JSON value must be a JSON object"));
    }

    let mut owned_value = value.clone();

    let event = owned_value
        .as_object_mut()
        .expect("safe since we checked above");

    let event_type_value = match event.get("type") {
        Some(event_type_value) => event_type_value,
        None => return Err(Error::new("Field `type` in JSON value must be present")),
    };

    let event_type = match event_type_value.as_str() {
        Some(event_type) => event_type.to_string(),
        None => {
            return Err(Error::new(
                "Field `type` in JSON value must be a JSON string",
            ))
        }
    };

    if let Some(content_value) = event.get_mut("content") {
        if !content_value.is_object() {
            return Err(Error::new(
                "Field `content` in JSON value must be a JSON object",
            ));
        }

        let content = content_value
            .as_object_mut()
            .expect("safe since we checked above");

        for key in content.clone().keys() {
            match event_type.as_ref() {
                "m.room.member" if key != "membership" => content.remove(key),
                "m.room.create" if key != "creator" => content.remove(key),
                "m.room.join_rules" if key != "join_rules" => content.remove(key),
                "m.room.power_levels" if !ALLOWED_POWER_LEVELS_KEYS.contains(&key.as_ref()) => {
                    content.remove(key)
                }
                "m.room.aliases" if key != "aliases" => content.remove(key),
                "m.room.history_visibility" if key != "history_visibility" => content.remove(key),
                _ => content.remove(key),
            };
        }
    }

    for key in event.clone().keys() {
        if !ALLOWED_KEYS.contains(&key.as_ref()) {
            event.remove(key);
        }
    }

    Ok(owned_value)
}
