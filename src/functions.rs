//! Functions for signing and verifying JSON and events.

use base64::{encode_config, STANDARD_NO_PAD};
use ring::digest::{digest, SHA256};
use serde_json::{from_str, map::Map, to_string, to_value, Value};

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

/// Signs an arbitrary JSON object and adds the signature to an object under the key `signatures`.
///
/// If `signatures` is already present, the new signature will be appended to the existing ones.
///
/// # Parameters
///
/// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
/// * key_pair: A cryptographic key pair used to sign the JSON.
/// * value: A JSON object to sign according and append a signature to.
///
/// # Errors
///
/// Returns an error if:
///
/// * `value` is not a JSON object.
/// * `value` contains a field called `signatures` that is not a JSON object.
/// * `server_name` cannot be parsed as a valid host.
pub fn sign_json<K>(server_name: &str, key_pair: &K, value: &mut Value) -> Result<(), Error>
where
    K: KeyPair,
{
    let mut signature_map;
    let maybe_unsigned;

    // Pull `signatures` and `unsigned` out of the object, and limit the scope of the mutable
    // borrow of `value` so we can call `to_string` with it below.
    {
        let map = match value {
            Value::Object(ref mut map) => map,
            _ => return Err(Error::new("JSON value must be a JSON object")),
        };

        signature_map = match map.remove("signatures") {
            Some(signatures_value) => match signatures_value.as_object() {
                Some(signatures) => from_str(&to_string(signatures)?)?,
                None => return Err(Error::new("Field `signatures` must be a JSON object")),
            },
            None => SignatureMap::with_capacity(1),
        };

        maybe_unsigned = map.remove("unsigned");
    }

    // Get the canonical JSON.
    let json = to_string(&value)?;

    // Sign the canonical JSON.
    let signature = key_pair.sign(json.as_bytes());

    // Insert the new signature in the map we pulled out (or created) previously.
    let signature_set = signature_map
        .entry(server_name)?
        .or_insert_with(|| SignatureSet::with_capacity(1));

    signature_set.insert(signature);

    // Safe to unwrap because we did this exact check at the beginning of the function.
    let map = value.as_object_mut().unwrap();

    // Put `signatures` and `unsigned` back in.
    map.insert("signatures".to_string(), to_value(signature_map)?);

    if let Some(unsigned) = maybe_unsigned {
        map.insert("unsigned".to_string(), to_value(unsigned)?);
    }

    Ok(())
}

/// Converts a JSON object into the "canonical" string form.
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
/// Returns the hash as a Base64-encoded string without padding.
///
/// The content hash of an event covers the complete event including the unredacted contents. It is
/// used during federation and is described in the Matrix server-server specification.
///
/// # Parameters
///
/// value: A JSON object to generate a content hash for.
///
/// # Errors
///
/// Returns an error if the provided JSON value is not a JSON object.
pub fn content_hash(value: &Value) -> Result<String, Error> {
    let json = to_canonical_json_with_fields_to_remove(value, CONTENT_HASH_FIELDS_TO_REMOVE)?;

    let hash = digest(&SHA256, json.as_bytes());

    Ok(encode_config(&hash, STANDARD_NO_PAD))
}

/// Creates a *reference hash* for the JSON representation of an event.
///
/// Returns the hash as a Base64-encoded string without padding.
///
/// The reference hash of an event covers the essential fields of an event, including content
/// hashes. It is used to generate event identifiers and is described in the Matrix server-server
/// specification.
///
/// # Parameters
///
/// value: A JSON object to generate a reference hash for.
///
/// # Errors
///
/// Returns an error if the provided JSON value is not a JSON object.
pub fn reference_hash(value: &Value) -> Result<String, Error> {
    let redacted_value = redact(value)?;

    let json =
        to_canonical_json_with_fields_to_remove(&redacted_value, REFERENCE_HASH_FIELDS_TO_REMOVE)?;

    let hash = digest(&SHA256, json.as_bytes());

    Ok(encode_config(&hash, STANDARD_NO_PAD))
}

/// Hashes and signs the JSON representation of an event and adds the hash and signature to objects
/// under the keys `hashes` and `signatures`, respectively.
///
/// If `hashes` and/or `signatures` are already present, the new data will be appended to the
/// existing data.
///
/// # Parameters
///
/// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
/// * key_pair: A cryptographic key pair used to sign the event.
/// * value: A JSON object to be hashed and signed according to the Matrix specification.
///
/// # Errors
///
/// Returns an error if:
///
/// * `value` is not a JSON object.
/// * `value` contains a field called `content` that is not a JSON object.
/// * `value` contains a field called `hashes` that is not a JSON object.
/// * `value` contains a field called `signatures` that is not a JSON object.
/// * `value` is missing the `type` field or the field is not a JSON string.
/// * `server_name` cannot be parsed as a valid host.
pub fn hash_and_sign_event<K>(
    server_name: &str,
    key_pair: &K,
    value: &mut Value,
) -> Result<(), Error>
where
    K: KeyPair,
{
    let hash = content_hash(value)?;

    // Limit the scope of the mutable borrow so `value` can be passed immutably to `redact` below.
    {
        let map = match value {
            Value::Object(ref mut map) => map,
            _ => return Err(Error::new("JSON value must be a JSON object")),
        };

        let hashes_value = map
            .entry("hashes")
            .or_insert_with(|| Value::Object(Map::with_capacity(1)));

        match hashes_value.as_object_mut() {
            Some(hashes) => hashes.insert("sha256".to_string(), Value::String(hash)),
            None => return Err(Error::new("Field `hashes` must be a JSON object")),
        };
    }

    let mut redacted = redact(value)?;

    sign_json(server_name, key_pair, &mut redacted)?;

    // Safe to unwrap because we did this exact check at the beginning of the function.
    let map = value.as_object_mut().unwrap();

    map.insert("signatures".to_string(), redacted["signatures"].take());

    Ok(())
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

    to_string(&owned_value).map_err(Error::from)
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
