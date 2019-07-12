//! Functions for signing and verifying JSON and events.

use std::collections::HashMap;

use base64::{decode_config, encode_config, STANDARD_NO_PAD};
use ring::digest::{digest, SHA256};
use serde_json::{from_str, from_value, map::Map, to_string, to_value, Value};

use crate::{
    keys::KeyPair,
    signatures::SignatureMap,
    verification::{Verified, Verifier},
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
///
/// # Examples
///
/// A homeserver signs JSON with a key pair:
///
/// ```rust
/// const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
/// const PRIVATE_KEY: &str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";
///
/// let public_key = base64::decode_config(&PUBLIC_KEY, base64::STANDARD_NO_PAD).unwrap();
/// let private_key = base64::decode_config(&PRIVATE_KEY, base64::STANDARD_NO_PAD).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = ruma_signatures::Ed25519KeyPair::new(
///     &public_key,
///     &private_key,
///     "1".to_string(), // The "version" of the key.
/// ).unwrap();
///
/// // Deserialize some JSON.
/// let mut value = serde_json::from_str("{}").unwrap();
///
/// // Sign the JSON with the key pair.
/// assert!(ruma_signatures::sign_json("example.com", &key_pair, &mut value).is_ok());
/// ```
///
/// This will modify the JSON from an empty object to a structure like this:
///
/// ```json
/// {
///     "signatures": {
///         "example.com": {
///             "ed25519:1": "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
///         }
///     }
/// }
/// ```
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
                Some(signatures) => from_value(Value::Object(signatures.clone()))?,
                None => return Err(Error::new("field `signatures` must be a JSON object")),
            },
            None => HashMap::with_capacity(1),
        };

        maybe_unsigned = map.remove("unsigned");
    }

    // Get the canonical JSON.
    let json = to_string(&value)?;

    // Sign the canonical JSON.
    let signature = key_pair.sign(json.as_bytes());

    // Insert the new signature in the map we pulled out (or created) previously.
    let signature_set = signature_map
        .entry(server_name.to_string())
        .or_insert_with(|| HashMap::with_capacity(1));

    signature_set.insert(signature.id(), signature.base64());

    // Safe to unwrap because we did this exact check at the beginning of the function.
    let map = value.as_object_mut().unwrap();

    // Put `signatures` and `unsigned` back in.
    map.insert("signatures".to_string(), to_value(signature_map)?);

    if let Some(unsigned) = maybe_unsigned {
        map.insert("unsigned".to_string(), to_value(unsigned)?);
    }

    Ok(())
}

/// Converts a JSON object into the
/// [canonical](https://matrix.org/docs/spec/appendices#canonical-json)  string form.
///
/// # Parameters
///
/// * value: The `serde_json::Value` (JSON value) to convert.
///
/// # Errors
///
/// Returns an error if the provided JSON value is not a JSON object.
pub fn canonical_json(value: &Value) -> Result<String, Error> {
    canonical_json_with_fields_to_remove(value, CANONICAL_JSON_FIELDS_TO_REMOVE)
}

/// Uses a set of public keys to verify a signed JSON object.
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
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
///
/// const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
/// const SIGNATURE_BYTES: &str =
///     "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ";
///
/// // Decode the public key used to generate the signature into raw bytes.
/// let public_key = base64::decode_config(&PUBLIC_KEY, base64::STANDARD_NO_PAD).unwrap();
///
/// // Create a `Signature` from the raw bytes of the signature.
/// let signature_bytes = base64::decode_config(&SIGNATURE_BYTES, base64::STANDARD_NO_PAD).unwrap();
/// let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).unwrap();
///
/// // Deserialize the signed JSON.
/// let value = serde_json::from_str(
///     r#"{
///         "signatures": {
///             "example.com": {
///                 "ed25519:1": "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
///             }
///         }
///     }"#
/// ).unwrap();
///
/// // Create the verifier for the Ed25519 algorithm.
/// let verifier = ruma_signatures::Ed25519Verifier;
///
/// // Create the `SignatureMap` that will inform `verify_json` which signatures to verify.
/// let mut signature_set = HashMap::new();
/// signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());
/// let mut verify_key_map = HashMap::new();
/// verify_key_map.insert("example.com".to_string(), signature_set);
///
/// // Verify at least one signature for each server in `verify_key_map`.
/// assert!(ruma_signatures::verify_json(&verifier, &verify_key_map, &value).is_ok());
/// ```
pub fn verify_json<V>(
    verifier: &V,
    verify_key_map: &SignatureMap,
    value: &Value,
) -> Result<(), Error>
where
    V: Verifier,
{
    let map = match value {
        Value::Object(ref map) => map,
        _ => return Err(Error::new("JSON value must be a JSON object")),
    };

    let signature_map: SignatureMap = match map.get("signatures") {
        Some(signatures_value) => match signatures_value.as_object() {
            Some(signatures) => from_value(Value::Object(signatures.clone()))?,
            None => return Err(Error::new("field `signatures` must be a JSON object")),
        },
        None => return Err(Error::new("JSON object must contain a `signatures` field.")),
    };

    for (server_name, verify_keys) in verify_key_map {
        let signature_set = match signature_map.get(server_name) {
            Some(set) => set,
            None => {
                return Err(Error::new(format!(
                    "no signatures found for server `{}`",
                    server_name
                )))
            }
        };

        let mut maybe_signature = None;
        let mut maybe_verify_key = None;

        for (key_id, verify_key) in verify_keys {
            if let Some(signature) = signature_set.get(key_id) {
                maybe_signature = Some(signature);
                maybe_verify_key = Some(verify_key);

                break;
            }
        }

        let signature = match maybe_signature {
            Some(signature) => signature,
            None => {
                return Err(Error::new(
                    "event is not signed with any of the given verify keys",
                ))
            }
        };

        let verify_key = match maybe_verify_key {
            Some(verify_key) => verify_key,
            None => {
                return Err(Error::new(
                    "event is not signed with any of the given verify keys",
                ))
            }
        };

        let signature_bytes = decode_config(signature, STANDARD_NO_PAD)?;

        let verify_key_bytes = decode_config(&verify_key, STANDARD_NO_PAD)?;

        verify_json_with(verifier, &verify_key_bytes, &signature_bytes, value)?;
    }

    Ok(())
}

/// Uses a public key to verify a signed JSON object.
///
/// # Parameters
///
/// * verifier: A `Verifier` appropriate for the digital signature algorithm that was used.
/// * public_key: The raw bytes of the public key used to sign the JSON.
/// * signature: The raw bytes of the signature.
/// * value: The `serde_json::Value` (JSON value) that was signed.
///
/// # Errors
///
/// Returns an error if:
///
/// * The provided JSON value is not a JSON object.
/// * Verification fails.
pub fn verify_json_with<V>(
    verifier: &V,
    public_key: &[u8],
    signature: &[u8],
    value: &Value,
) -> Result<(), Error>
where
    V: Verifier,
{
    verifier.verify_json(public_key, signature, canonical_json(value)?.as_bytes())
}

/// Creates a *content hash* for the JSON representation of an event.
///
/// Returns the hash as a Base64-encoded string, using the standard character set, without padding.
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
    let json = canonical_json_with_fields_to_remove(value, CONTENT_HASH_FIELDS_TO_REMOVE)?;

    let hash = digest(&SHA256, json.as_bytes());

    Ok(encode_config(&hash, STANDARD_NO_PAD))
}

/// Creates a *reference hash* for the JSON representation of an event.
///
/// Returns the hash as a Base64-encoded string, using the standard character set, without padding.
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
        canonical_json_with_fields_to_remove(&redacted_value, REFERENCE_HASH_FIELDS_TO_REMOVE)?;

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
///
/// # Examples
///
/// ```rust
/// const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
/// const PRIVATE_KEY: &str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";
///
/// let public_key = base64::decode_config(&PUBLIC_KEY, base64::STANDARD_NO_PAD).unwrap();
/// let private_key = base64::decode_config(&PRIVATE_KEY, base64::STANDARD_NO_PAD).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = ruma_signatures::Ed25519KeyPair::new(
///     &public_key,
///     &private_key,
///     "1".to_string(), // The "version" of the key.
/// ).unwrap();
///
/// // Deserialize an event from JSON.
/// let mut value = serde_json::from_str(
///     r#"{
///         "room_id": "!x:domain",
///         "sender": "@a:domain",
///         "origin": "domain",
///         "origin_server_ts": 1000000,
///         "signatures": {},
///         "hashes": {},
///         "type": "X",
///         "content": {},
///         "prev_events": [],
///         "auth_events": [],
///         "depth": 3,
///         "unsigned": {
///             "age_ts": 1000000
///         }
///     }"#
/// ).unwrap();
///
/// // Hash and sign the JSON with the key pair.
/// assert!(ruma_signatures::hash_and_sign_event("example.com", &key_pair, &mut value).is_ok());
/// ```
///
/// This will modify the JSON from the structure shown to a structure like this:
///
/// ```json
/// {
///     "auth_events": [],
///     "content": {},
///     "depth": 3,
///     "hashes": {
///         "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
///     },
///     "origin": "domain",
///     "origin_server_ts": 1000000,
///     "prev_events": [],
///     "room_id": "!x:domain",
///     "sender": "@a:domain",
///     "signatures": {
///         "domain": {
///             "ed25519:1": "KxwGjPSDEtvnFgU00fwFz+l6d2pJM6XBIaMEn81SXPTRl16AqLAYqfIReFGZlHi5KLjAWbOoMszkwsQma+lYAg"
///         }
///     },
///     "type": "X",
///     "unsigned": {
///         "age_ts": 1000000
///     }
/// }
/// ```
///
/// Notice the addition of `hashes` and `signatures`.
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
            None => return Err(Error::new("field `hashes` must be a JSON object")),
        };
    }

    let mut redacted = redact(value)?;

    sign_json(server_name, key_pair, &mut redacted)?;

    // Safe to unwrap because we did this exact check at the beginning of the function.
    let map = value.as_object_mut().unwrap();

    map.insert("signatures".to_string(), redacted["signatures"].take());

    Ok(())
}

/// Uses a set of public keys to verify a signed JSON representation of an event.
///
/// Some room versions may require signatures from multiple homeservers, so this function takes a
/// map from servers to sets of public keys. For each homeserver present in the map, this function
/// will require a valid signature. All known public keys for a homeserver should be provided. The
/// first one found on the given event will be used.
///
/// If the `Ok` variant is returned by this function, it will contain a `Verified` value which
/// distinguishes an event with valid signatures and a matching content hash with an event with
/// only valid signatures. See the documetation for `Verified` for details.
///
/// # Parameters
///
/// * verifier: A `Verifier` appropriate for the digital signature algorithm that was used.
/// * verify_key_map: A map from server names to a map from key identifiers to public keys. Server
/// names are the hostname or IP of a homeserver (e.g. "example.com") for which a signature must be
/// verified. Key identifiers for each server (e.g. "ed25519:1") then map to their respective public
/// keys.
/// * value: The `serde_json::Value` (JSON value) of the event that was signed.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
///
/// const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
///
/// // Deserialize an event from JSON.
/// let value = serde_json::from_str(
///     r#"{
///         "auth_events": [],
///         "content": {},
///         "depth": 3,
///         "hashes": {
///             "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
///         },
///         "origin": "domain",
///         "origin_server_ts": 1000000,
///         "prev_events": [],
///         "room_id": "!x:domain",
///         "sender": "@a:domain",
///         "signatures": {
///             "domain": {
///                 "ed25519:1": "KxwGjPSDEtvnFgU00fwFz+l6d2pJM6XBIaMEn81SXPTRl16AqLAYqfIReFGZlHi5KLjAWbOoMszkwsQma+lYAg"
///             }
///         },
///         "type": "X",
///         "unsigned": {
///             "age_ts": 1000000
///         }
///     }"#
/// ).unwrap();
///
/// // Create the verifier for the Ed25519 algorithm.
/// let verifier = ruma_signatures::Ed25519Verifier;
///
/// // Create a map from key ID to public key.
/// let mut example_server_keys = HashMap::new();
/// example_server_keys.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());
///
/// // Insert the public keys into a map keyed by server name.
/// let mut verify_key_map = HashMap::new();
/// verify_key_map.insert("domain".to_string(), example_server_keys);
///
/// // Verify at least one signature for each server in `verify_key_map`.
/// assert!(ruma_signatures::verify_event(&verifier, &verify_key_map, &value).is_ok());
/// ```
pub fn verify_event<V>(
    verifier: &V,
    verify_key_map: &SignatureMap,
    value: &Value,
) -> Result<Verified, Error>
where
    V: Verifier,
{
    let redacted = redact(value)?;

    let map = match redacted {
        Value::Object(ref map) => map,
        _ => return Err(Error::new("JSON value must be a JSON object")),
    };

    let hash = match map.get("hashes") {
        Some(hashes_value) => match hashes_value.as_object() {
            Some(hashes) => match hashes.get("sha256") {
                Some(hash_value) => match hash_value.as_str() {
                    Some(hash) => hash,
                    None => return Err(Error::new("sha256 hash must be a JSON string")),
                },
                None => return Err(Error::new("field `hashes` must be a JSON object")),
            },
            None => return Err(Error::new("event missing sha256 hash")),
        },
        None => return Err(Error::new("field `hashes` must be present")),
    };

    let signature_map: SignatureMap = match map.get("signatures") {
        Some(signatures_value) => match signatures_value.as_object() {
            Some(signatures) => from_value(Value::Object(signatures.clone()))?,
            None => return Err(Error::new("field `signatures` must be a JSON object")),
        },
        None => return Err(Error::new("JSON object must contain a `signatures` field.")),
    };

    for (server_name, verify_keys) in verify_key_map {
        let signature_set = match signature_map.get(server_name) {
            Some(set) => set,
            None => {
                return Err(Error::new(format!(
                    "no signatures found for server `{}`",
                    server_name
                )))
            }
        };

        let mut maybe_signature = None;
        let mut maybe_verify_key = None;

        for (key_id, verify_key) in verify_keys {
            if let Some(signature) = signature_set.get(key_id) {
                maybe_signature = Some(signature);
                maybe_verify_key = Some(verify_key);

                break;
            }
        }

        let signature = match maybe_signature {
            Some(signature) => signature,
            None => {
                return Err(Error::new(
                    "event is not signed with any of the given verify keys",
                ))
            }
        };

        let verify_key = match maybe_verify_key {
            Some(verify_key) => verify_key,
            None => {
                return Err(Error::new(
                    "event is not signed with any of the given verify keys",
                ))
            }
        };

        let canonical_json = from_str(&canonical_json(&redacted)?)?;

        let signature_bytes = decode_config(signature, STANDARD_NO_PAD)?;

        let verify_key_bytes = decode_config(&verify_key, STANDARD_NO_PAD)?;

        verify_json_with(
            verifier,
            &verify_key_bytes,
            &signature_bytes,
            &canonical_json,
        )?;
    }

    let calculated_hash = content_hash(value)?;

    if hash == calculated_hash {
        Ok(Verified::All)
    } else {
        Ok(Verified::Signatures)
    }
}

/// Internal implementation detail of the canonical JSON algorithm. Allows customization of the
/// fields that will be removed before serializing.
fn canonical_json_with_fields_to_remove(value: &Value, fields: &[&str]) -> Result<String, Error> {
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

/// Redacts the JSON representation of an event using the rules specified in the Matrix
/// client-server specification.
///
/// This is part of the process of signing an event.
///
/// Redaction is also suggested when a verifying an event with `verify_event` returns
/// `Verified::Signatures`. See the documentation for `Verified` for details.
///
/// Returns a new `serde_json::Value` with all applicable fields redacted.
///
/// # Parameters
///
/// * value: A JSON object to redact.
pub fn redact(value: &Value) -> Result<Value, Error> {
    if !value.is_object() {
        return Err(Error::new("JSON value must be a JSON object"));
    }

    let mut owned_value = value.clone();

    let event = owned_value
        .as_object_mut()
        .expect("safe since we checked above");

    let event_type_value = match event.get("type") {
        Some(event_type_value) => event_type_value,
        None => return Err(Error::new("field `type` in JSON value must be present")),
    };

    let event_type = match event_type_value.as_str() {
        Some(event_type) => event_type.to_string(),
        None => {
            return Err(Error::new(
                "field `type` in JSON value must be a JSON string",
            ))
        }
    };

    if let Some(content_value) = event.get_mut("content") {
        let map = match content_value {
            Value::Object(ref mut map) => map,
            _ => {
                return Err(Error::new(
                    "field `content` in JSON value must be a JSON object",
                ))
            }
        };

        for key in map.clone().keys() {
            match event_type.as_ref() {
                "m.room.member" if key != "membership" => map.remove(key),
                "m.room.create" if key != "creator" => map.remove(key),
                "m.room.join_rules" if key != "join_rules" => map.remove(key),
                "m.room.power_levels" if !ALLOWED_POWER_LEVELS_KEYS.contains(&key.as_ref()) => {
                    map.remove(key)
                }
                "m.room.aliases" if key != "aliases" => map.remove(key),
                "m.room.history_visibility" if key != "history_visibility" => map.remove(key),
                _ => map.remove(key),
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
