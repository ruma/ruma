//! Functions for signing and verifying JSON and events.

use std::collections::HashMap;

use base64::{decode_config, encode_config, STANDARD_NO_PAD};
use ring::digest::{digest, SHA256};
use serde_json::{from_str, from_value, map::Map, to_string, to_value, Value};

use crate::{
    keys::{KeyPair, PublicKeyMap},
    signatures::SignatureMap,
    split_id,
    verification::{Ed25519Verifier, Verified, Verifier},
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
/// * entity_id: The identifier of the entity creating the signature. Generally this means a
/// homeserver, e.g. "example.com".
/// * key_pair: A cryptographic key pair used to sign the JSON.
/// * value: A JSON object to sign according and append a signature to.
///
/// # Errors
///
/// Returns an error if:
///
/// * `value` is not a JSON object.
/// * `value` contains a field called `signatures` that is not a JSON object.
///
/// # Examples
///
/// A homeserver signs JSON with a key pair:
///
/// ```rust
/// const PKCS8: &str = "\
///     MFMCAQEwBQYDK2VwBCIEINjozvdfbsGEt6DD+7Uf4PiJ/YvTNXV2mIPc/\
///     tA0T+6toSMDIQDdM+tpNzNWQM9NFpfgr4B9S7LHszOrVRp9NfKmeXS3aQ\
/// ";
///
/// let document = base64::decode_config(&PKCS8, base64::STANDARD_NO_PAD).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = ruma_signatures::Ed25519KeyPair::new(
///     &document,
///     "1".into(), // The "version" of the key.
/// ).unwrap();
///
/// // Deserialize some JSON.
/// let mut value = serde_json::from_str("{}").unwrap();
///
/// // Sign the JSON with the key pair.
/// assert!(ruma_signatures::sign_json("domain", &key_pair, &mut value).is_ok());
/// ```
///
/// This will modify the JSON from an empty object to a structure like this:
///
/// ```json
/// {
///     "signatures": {
///         "domain": {
///             "ed25519:1": "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
///         }
///     }
/// }
/// ```
pub fn sign_json<K>(entity_id: &str, key_pair: &K, value: &mut Value) -> Result<(), Error>
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
    let signature_set =
        signature_map.entry(entity_id.to_string()).or_insert_with(|| HashMap::with_capacity(1));

    signature_set.insert(signature.id(), signature.base64());

    // Safe to unwrap because we did this exact check at the beginning of the function.
    let map = value.as_object_mut().unwrap();

    // Put `signatures` and `unsigned` back in.
    map.insert("signatures".into(), to_value(signature_map)?);

    if let Some(unsigned) = maybe_unsigned {
        map.insert("unsigned".into(), to_value(unsigned)?);
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
///
/// # Examples
///
/// ```rust
/// let input =
///     r#"{
///         "本": 2,
///         "日": 1
///     }"#;
///
/// let value = serde_json::from_str::<serde_json::Value>(input).unwrap();
///
/// let canonical = ruma_signatures::canonical_json(&value).unwrap();
///
/// assert_eq!(canonical, r#"{"日":1,"本":2}"#);
/// ```
pub fn canonical_json(value: &Value) -> Result<String, Error> {
    canonical_json_with_fields_to_remove(value, CANONICAL_JSON_FIELDS_TO_REMOVE)
}

/// Uses a set of public keys to verify a signed JSON object.
///
/// # Parameters
///
/// * public_key_map: A map from entity identifiers to a map from key identifiers to public keys.
/// Generally, entity identifiers are server names—the host/IP/port of a homeserver (e.g.
/// "example.com") for which a signature must be verified. Key identifiers for each server (e.g.
/// "ed25519:1") then map to their respective public keys.
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
///
/// // Deserialize the signed JSON.
/// let value = serde_json::from_str(
///     r#"{
///         "signatures": {
///             "domain": {
///                 "ed25519:1": "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
///             }
///         }
///     }"#
/// ).unwrap();
///
/// // Create the `PublicKeyMap` that will inform `verify_json` which signatures to verify.
/// let mut public_key_set = HashMap::new();
/// public_key_set.insert("ed25519:1".into(), PUBLIC_KEY.to_string());
/// let mut public_key_map = HashMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// assert!(ruma_signatures::verify_json(&public_key_map, &value).is_ok());
/// ```
pub fn verify_json(public_key_map: &PublicKeyMap, value: &Value) -> Result<(), Error> {
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

    for (entity_id, public_keys) in public_key_map {
        let signature_set = match signature_map.get(entity_id) {
            Some(set) => set,
            None => {
                return Err(Error::new(format!("no signatures found for entity `{}`", entity_id)))
            }
        };

        let mut maybe_signature = None;
        let mut maybe_public_key = None;

        for (key_id, public_key) in public_keys {
            // Since only ed25519 is supported right now, we don't actually need to check what the
            // algorithm is. If it split successfully, it's ed25519.
            if split_id(key_id).is_err() {
                break;
            }

            if let Some(signature) = signature_set.get(key_id) {
                maybe_signature = Some(signature);
                maybe_public_key = Some(public_key);

                break;
            }
        }

        let signature = match maybe_signature {
            Some(signature) => signature,
            None => {
                return Err(Error::new("event is not signed with any of the given public keys"))
            }
        };

        let public_key = match maybe_public_key {
            Some(public_key) => public_key,
            None => {
                return Err(Error::new("event is not signed with any of the given public keys"))
            }
        };

        let signature_bytes = decode_config(signature, STANDARD_NO_PAD)?;

        let public_key_bytes = decode_config(&public_key, STANDARD_NO_PAD)?;

        verify_json_with(&Ed25519Verifier, &public_key_bytes, &signature_bytes, value)?;
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
fn verify_json_with<V>(
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
/// * entity_id: The identifier of the entity creating the signature. Generally this means a
/// homeserver, e.g. "example.com".
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
///
/// # Examples
///
/// ```rust
/// const PKCS8: &str = "\
///     MFMCAQEwBQYDK2VwBCIEINjozvdfbsGEt6DD+7Uf4PiJ/YvTNXV2mIPc/\
///     tA0T+6toSMDIQDdM+tpNzNWQM9NFpfgr4B9S7LHszOrVRp9NfKmeXS3aQ\
/// ";
///
/// let document = base64::decode_config(&PKCS8, base64::STANDARD_NO_PAD).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = ruma_signatures::Ed25519KeyPair::new(
///     &document,
///     "1".into(), // The "version" of the key.
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
/// assert!(ruma_signatures::hash_and_sign_event("domain", &key_pair, &mut value).is_ok());
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
pub fn hash_and_sign_event<K>(entity_id: &str, key_pair: &K, value: &mut Value) -> Result<(), Error>
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

        let hashes_value =
            map.entry("hashes").or_insert_with(|| Value::Object(Map::with_capacity(1)));

        match hashes_value.as_object_mut() {
            Some(hashes) => hashes.insert("sha256".into(), Value::String(hash)),
            None => return Err(Error::new("field `hashes` must be a JSON object")),
        };
    }

    let mut redacted = redact(value)?;

    sign_json(entity_id, key_pair, &mut redacted)?;

    // Safe to unwrap because we did this exact check at the beginning of the function.
    let map = value.as_object_mut().unwrap();

    map.insert("signatures".into(), redacted["signatures"].take());

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
/// * public_key_map: A map from entity identifiers to a map from key identifiers to public keys.
/// Generally, entity identifiers are server names—the host/IP/port of a homeserver (e.g.
/// "example.com") for which a signature must be verified. Key identifiers for each server (e.g.
/// "ed25519:1") then map to their respective public keys.
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
/// // Create the `PublicKeyMap` that will inform `verify_json` which signatures to verify.
/// let mut public_key_set = HashMap::new();
/// public_key_set.insert("ed25519:1".into(), PUBLIC_KEY.to_string());
/// let mut public_key_map = HashMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// assert!(ruma_signatures::verify_event(&public_key_map, &value).is_ok());
/// ```
pub fn verify_event(public_key_map: &PublicKeyMap, value: &Value) -> Result<Verified, Error> {
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

    for (entity_id, public_keys) in public_key_map {
        let signature_set = match signature_map.get(entity_id) {
            Some(set) => set,
            None => {
                return Err(Error::new(format!("no signatures found for entity `{}`", entity_id)))
            }
        };

        let mut maybe_signature = None;
        let mut maybe_public_key = None;

        for (key_id, public_key) in public_keys {
            // Since only ed25519 is supported right now, we don't actually need to check what the
            // algorithm is. If it split successfully, it's ed25519.
            if split_id(key_id).is_err() {
                break;
            }

            if let Some(signature) = signature_set.get(key_id) {
                maybe_signature = Some(signature);
                maybe_public_key = Some(public_key);

                break;
            }
        }

        let signature = match maybe_signature {
            Some(signature) => signature,
            None => {
                return Err(Error::new("event is not signed with any of the given public keys"))
            }
        };

        let public_key = match maybe_public_key {
            Some(public_key) => public_key,
            None => {
                return Err(Error::new("event is not signed with any of the given public keys"))
            }
        };

        let canonical_json = from_str(&canonical_json(&redacted)?)?;

        let signature_bytes = decode_config(signature, STANDARD_NO_PAD)?;

        let public_key_bytes = decode_config(&public_key, STANDARD_NO_PAD)?;

        verify_json_with(&Ed25519Verifier, &public_key_bytes, &signature_bytes, &canonical_json)?;
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
        let object = owned_value.as_object_mut().expect("safe since we checked above");

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
pub fn redact(value: &Value) -> Result<Value, Error> {
    if !value.is_object() {
        return Err(Error::new("JSON value must be a JSON object"));
    }

    let mut owned_value = value.clone();

    let event = owned_value.as_object_mut().expect("safe since we checked above");

    let event_type_value = match event.get("type") {
        Some(event_type_value) => event_type_value,
        None => return Err(Error::new("field `type` in JSON value must be present")),
    };

    let event_type = match event_type_value.as_str() {
        Some(event_type) => event_type.to_string(),
        None => return Err(Error::new("field `type` in JSON value must be a JSON string")),
    };

    if let Some(content_value) = event.get_mut("content") {
        let map = match content_value {
            Value::Object(ref mut map) => map,
            _ => return Err(Error::new("field `content` in JSON value must be a JSON object")),
        };

        for key in map.clone().keys() {
            match event_type.as_ref() {
                "m.room.member" => {
                    if key != "membership" {
                        map.remove(key);
                    }
                }
                "m.room.create" => {
                    if key != "creator" {
                        map.remove(key);
                    }
                }
                "m.room.join_rules" => {
                    if key != "join_rules" {
                        map.remove(key);
                    }
                }
                "m.room.power_levels" => {
                    if !ALLOWED_POWER_LEVELS_KEYS.contains(&key.as_ref()) {
                        map.remove(key);
                    }
                }
                "m.room.aliases" => {
                    if key != "aliases" {
                        map.remove(key);
                    }
                }
                "m.room.history_visibility" => {
                    if key != "history_visibility" {
                        map.remove(key);
                    }
                }
                _ => {
                    map.remove(key);
                }
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::canonical_json;

    #[test]
    fn canonical_json_complex() {
        let data = json!({
            "auth": {
                "success": true,
                "mxid": "@john.doe:example.com",
                "profile": {
                    "display_name": "John Doe",
                    "three_pids": [
                        {
                            "medium": "email",
                            "address": "john.doe@example.org"
                        },
                        {
                            "medium": "msisdn",
                            "address": "123456789"
                        }
                    ]
                }
            }
        });

        let canonical = r#"{"auth":{"mxid":"@john.doe:example.com","profile":{"display_name":"John Doe","three_pids":[{"address":"john.doe@example.org","medium":"email"},{"address":"123456789","medium":"msisdn"}]},"success":true}}"#;

        assert_eq!(canonical_json(&data).unwrap(), canonical);
    }
}
