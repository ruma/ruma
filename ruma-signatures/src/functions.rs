//! Functions for signing and verifying JSON and events.

use std::{collections::BTreeMap, mem};

use base64::{decode_config, encode_config, STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use ring::digest::{digest, SHA256};
use ruma_identifiers::RoomVersionId;
use ruma_serde::{to_canonical_json_string, CanonicalJsonObject, CanonicalJsonValue};
use serde_json::from_str as from_json_str;

use crate::{
    keys::{KeyPair, PublicKeyMap},
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

fn allowed_content_keys_for(event_type: &str, version: &RoomVersionId) -> &'static [&'static str] {
    match event_type {
        "m.room.member" => &["membership"],
        "m.room.create" => &["creator"],
        "m.room.join_rules" => &["join_rule"],
        "m.room.power_levels" => &[
            "ban",
            "events",
            "events_default",
            "kick",
            "redact",
            "state_default",
            "users",
            "users_default",
        ],
        "m.room.aliases" => match version {
            RoomVersionId::Version1
            | RoomVersionId::Version2
            | RoomVersionId::Version3
            | RoomVersionId::Version4
            | RoomVersionId::Version5 => &["join_rule"],
            // All other room versions, including custom ones, are treated by version 6 rules.
            // TODO: Should we return an error for unknown versions instead?
            _ => &[],
        },
        "m.room.history_visibility" => &["history_visibility"],
        _ => &[],
    }
}

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
/// * object: A JSON object to sign according and append a signature to.
///
/// # Errors
///
/// Returns an error if:
///
/// * `object` contains a field called `signatures` that is not a JSON object.
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
pub fn sign_json<K>(
    entity_id: &str,
    key_pair: &K,
    object: &mut CanonicalJsonObject,
) -> Result<(), Error>
where
    K: KeyPair,
{
    let mut signature_map;
    let maybe_unsigned;

    // FIXME: Once MSRV >= 1.45.0, use remove_key and don't allocate new `String`s below.
    signature_map = match object.remove("signatures") {
        Some(CanonicalJsonValue::Object(signatures)) => signatures,
        Some(_) => return Err(Error::new("field `signatures` must be a JSON object")),
        None => BTreeMap::new(),
    };

    maybe_unsigned = object.remove("unsigned");

    // Get the canonical JSON string.
    let json = to_canonical_json_string(object)?;

    // Sign the canonical JSON string.
    let signature = key_pair.sign(json.as_bytes());

    // Insert the new signature in the map we pulled out (or created) previously.
    let signature_set = signature_map
        .entry(entity_id.to_string())
        .or_insert_with(|| CanonicalJsonValue::Object(BTreeMap::new()));

    let signature_set = match signature_set {
        CanonicalJsonValue::Object(obj) => obj,
        _ => return Err(Error::new("fields in `signatures` must be objects")),
    };

    signature_set.insert(signature.id(), CanonicalJsonValue::String(signature.base64()));

    // Put `signatures` and `unsigned` back in.
    object.insert("signatures".into(), CanonicalJsonValue::Object(signature_map));

    if let Some(unsigned) = maybe_unsigned {
        object.insert("unsigned".into(), unsigned);
    }

    Ok(())
}

/// Converts an event into the [canonical] string form.
///
/// [canonical]: https://matrix.org/docs/spec/appendices#canonical-json
///
/// # Parameters
///
/// * object: The JSON object to convert.
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
/// let object = serde_json::from_str(input).unwrap();
/// let canonical = ruma_signatures::canonical_json(&object);
///
/// assert_eq!(canonical, r#"{"日":1,"本":2}"#);
/// ```
pub fn canonical_json(object: &CanonicalJsonObject) -> String {
    canonical_json_with_fields_to_remove(object, CANONICAL_JSON_FIELDS_TO_REMOVE)
}

/// Uses a set of public keys to verify a signed JSON object.
///
/// # Parameters
///
/// * public_key_map: A map from entity identifiers to a map from key identifiers to public keys.
/// Generally, entity identifiers are server names—the host/IP/port of a homeserver (e.g.
/// "example.com") for which a signature must be verified. Key identifiers for each server (e.g.
/// "ed25519:1") then map to their respective public keys.
/// * object: The JSON object that was signed.
///
/// # Errors
///
/// Returns an error if verification fails.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
///
/// const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
///
/// // Deserialize the signed JSON.
/// let object = serde_json::from_str(
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
/// let mut public_key_set = BTreeMap::new();
/// public_key_set.insert("ed25519:1".into(), PUBLIC_KEY.to_string());
/// let mut public_key_map = BTreeMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// assert!(ruma_signatures::verify_json(&public_key_map, &object).is_ok());
/// ```
pub fn verify_json(
    public_key_map: &PublicKeyMap,
    object: &CanonicalJsonObject,
) -> Result<(), Error> {
    let signature_map = match object.get("signatures") {
        Some(CanonicalJsonValue::Object(signatures)) => signatures.clone(),
        Some(_) => return Err(Error::new("field `signatures` must be a JSON object")),
        None => return Err(Error::new("JSON object must contain a `signatures` field.")),
    };

    for (entity_id, public_keys) in public_key_map {
        let signature_set = match signature_map.get(entity_id) {
            Some(CanonicalJsonValue::Object(set)) => set,
            Some(_) => return Err(Error::new("signature sets must be JSON objects")),
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
            Some(CanonicalJsonValue::String(signature)) => signature,
            Some(_) => return Err(Error::new("signature must be a string")),
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

        verify_json_with(&Ed25519Verifier, &public_key_bytes, &signature_bytes, object)?;
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
/// * object: The JSON object that was signed.
///
/// # Errors
///
/// Returns an error if verification fails.
fn verify_json_with<V>(
    verifier: &V,
    public_key: &[u8],
    signature: &[u8],
    object: &CanonicalJsonObject,
) -> Result<(), Error>
where
    V: Verifier,
{
    verifier.verify_json(public_key, signature, canonical_json(object).as_bytes())
}

/// Creates a *content hash* for an event.
///
/// Returns the hash as a Base64-encoded string, using the standard character set, without padding.
///
/// The content hash of an event covers the complete event including the unredacted contents. It is
/// used during federation and is described in the Matrix server-server specification.
///
/// # Parameters
///
/// object: A JSON object to generate a content hash for.
pub fn content_hash(object: &CanonicalJsonObject) -> String {
    let json = canonical_json_with_fields_to_remove(object, CONTENT_HASH_FIELDS_TO_REMOVE);
    let hash = digest(&SHA256, json.as_bytes());

    encode_config(&hash, STANDARD_NO_PAD)
}

/// Creates a *reference hash* for an event.
///
/// Returns the hash as a Base64-encoded string, using the standard character set, without padding.
///
/// The reference hash of an event covers the essential fields of an event, including content
/// hashes. It is used to generate event identifiers and is described in the Matrix server-server
/// specification.
///
/// # Parameters
///
/// object: A JSON object to generate a reference hash for.
///
/// # Errors
///
/// Returns an error if redaction fails.
pub fn reference_hash(
    value: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<String, Error> {
    let redacted_value = redact(value, version)?;

    let json =
        canonical_json_with_fields_to_remove(&redacted_value, REFERENCE_HASH_FIELDS_TO_REMOVE);

    let hash = digest(&SHA256, json.as_bytes());

    Ok(encode_config(
        &hash,
        match version {
            RoomVersionId::Version1 | RoomVersionId::Version2 | RoomVersionId::Version3 => {
                STANDARD_NO_PAD
            }
            // Room versions higher than version 3 are url safe base64 encoded
            _ => URL_SAFE_NO_PAD,
        },
    ))
}

/// Hashes and signs an event and adds the hash and signature to objects under the keys `hashes` and
/// `signatures`, respectively.
///
/// If `hashes` and/or `signatures` are already present, the new data will be appended to the
/// existing data.
///
/// # Parameters
///
/// * entity_id: The identifier of the entity creating the signature. Generally this means a
/// homeserver, e.g. "example.com".
/// * key_pair: A cryptographic key pair used to sign the event.
/// * object: A JSON object to be hashed and signed according to the Matrix specification.
///
/// # Errors
///
/// Returns an error if:
///
/// * `object` contains a field called `content` that is not a JSON object.
/// * `object` contains a field called `hashes` that is not a JSON object.
/// * `object` contains a field called `signatures` that is not a JSON object.
/// * `object` is missing the `type` field or the field is not a JSON string.
///
/// # Examples
///
/// ```rust
/// # use ruma_identifiers::RoomVersionId;
/// # use ruma_signatures::{hash_and_sign_event, Ed25519KeyPair};
/// #
/// const PKCS8: &str = "\
///     MFMCAQEwBQYDK2VwBCIEINjozvdfbsGEt6DD+7Uf4PiJ/YvTNXV2mIPc/\
///     tA0T+6toSMDIQDdM+tpNzNWQM9NFpfgr4B9S7LHszOrVRp9NfKmeXS3aQ\
/// ";
///
/// let document = base64::decode_config(&PKCS8, base64::STANDARD_NO_PAD).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = Ed25519KeyPair::new(
///     &document,
///     "1".into(), // The "version" of the key.
/// ).unwrap();
///
/// // Deserialize an event from JSON.
/// let mut object = serde_json::from_str(
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
/// assert!(hash_and_sign_event("domain", &key_pair, &mut object, &RoomVersionId::Version1).is_ok());
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
    entity_id: &str,
    key_pair: &K,
    object: &mut CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<(), Error>
where
    K: KeyPair,
{
    let hash = content_hash(object);

    let hashes_value = object
        .entry("hashes".to_owned())
        .or_insert_with(|| CanonicalJsonValue::Object(BTreeMap::new()));

    match hashes_value {
        CanonicalJsonValue::Object(hashes) => {
            hashes.insert("sha256".into(), CanonicalJsonValue::String(hash))
        }
        _ => return Err(Error::new("field `hashes` must be a JSON object")),
    };

    let mut redacted = redact(object, version)?;

    sign_json(entity_id, key_pair, &mut redacted)?;

    object.insert("signatures".into(), mem::take(redacted.get_mut("signatures").unwrap()));

    Ok(())
}

/// Uses a set of public keys to verify a signed event.
///
/// Some room versions may require signatures from multiple homeservers, so this function takes a
/// map from servers to sets of public keys. For each homeserver present in the map, this function
/// will require a valid signature. All known public keys for a homeserver should be provided. The
/// first one found on the given event will be used.
///
/// If the `Ok` variant is returned by this function, it will contain a `Verified` value which
/// distinguishes an event with valid signatures and a matching content hash with an event with
/// only valid signatures. See the documentation for `Verified` for details.
///
/// # Parameters
///
/// * public_key_map: A map from entity identifiers to a map from key identifiers to public keys.
/// Generally, entity identifiers are server names—the host/IP/port of a homeserver (e.g.
/// "example.com") for which a signature must be verified. Key identifiers for each server (e.g.
/// "ed25519:1") then map to their respective public keys.
/// * object: The JSON object of the event that was signed.
///
/// # Examples
///
/// ```rust
/// # use std::collections::BTreeMap;
/// # use ruma_identifiers::RoomVersionId;
/// # use ruma_signatures::verify_event;
/// #
/// const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
///
/// // Deserialize an event from JSON.
/// let object = serde_json::from_str(
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
/// let mut public_key_set = BTreeMap::new();
/// public_key_set.insert("ed25519:1".into(), PUBLIC_KEY.to_string());
/// let mut public_key_map = BTreeMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// assert!(verify_event(&public_key_map, &object, &RoomVersionId::Version6).is_ok());
/// ```
pub fn verify_event(
    public_key_map: &PublicKeyMap,
    object: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<Verified, Error> {
    let redacted = redact(object, version)?;

    let hash = match object.get("hashes") {
        Some(hashes_value) => match hashes_value {
            CanonicalJsonValue::Object(hashes) => match hashes.get("sha256") {
                Some(hash_value) => match hash_value {
                    CanonicalJsonValue::String(hash) => hash,
                    _ => return Err(Error::new("sha256 hash must be a JSON string")),
                },
                None => return Err(Error::new("field `hashes` must be a JSON object")),
            },
            _ => return Err(Error::new("event missing sha256 hash")),
        },
        None => return Err(Error::new("field `hashes` must be present")),
    };

    let signature_map = match object.get("signatures") {
        Some(CanonicalJsonValue::Object(signatures)) => signatures,
        Some(_) => return Err(Error::new("field `signatures` must be a JSON object")),
        None => return Err(Error::new("JSON object must contain a `signatures` field.")),
    };

    for (entity_id, public_keys) in public_key_map {
        let signature_set = match signature_map.get(entity_id) {
            Some(CanonicalJsonValue::Object(set)) => set,
            Some(_) => return Err(Error::new("signatures sets must be JSON objects")),
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
            Some(CanonicalJsonValue::String(signature)) => signature,
            Some(_) => return Err(Error::new("signature must be a string")),
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

        let canonical_json = from_json_str(&canonical_json(&redacted))?;

        let signature_bytes = decode_config(signature, STANDARD_NO_PAD)?;

        let public_key_bytes = decode_config(&public_key, STANDARD_NO_PAD)?;

        verify_json_with(&Ed25519Verifier, &public_key_bytes, &signature_bytes, &canonical_json)?;
    }

    let calculated_hash = content_hash(object);

    if *hash == calculated_hash {
        Ok(Verified::All)
    } else {
        Ok(Verified::Signatures)
    }
}

/// Internal implementation detail of the canonical JSON algorithm. Allows customization of the
/// fields that will be removed before serializing.
fn canonical_json_with_fields_to_remove(object: &CanonicalJsonObject, fields: &[&str]) -> String {
    let mut owned_object = object.clone();

    for field in fields {
        owned_object.remove(*field);
    }

    to_canonical_json_string(&owned_object).expect("JSON object serialization to succeed")
}

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// This is part of the process of signing an event.
///
/// Redaction is also suggested when a verifying an event with `verify_event` returns
/// `Verified::Signatures`. See the documentation for `Verified` for details.
///
/// Returns a new JSON object with all applicable fields redacted.
///
/// # Parameters
///
/// * object: A JSON object to redact.
///
/// # Errors
///
/// Returns an error if:
///
/// * `object` contains a field called `content` that is not a JSON object.
/// * `object` contains a field called `hashes` that is not a JSON object.
/// * `object` contains a field called `signatures` that is not a JSON object.
/// * `object` is missing the `type` field or the field is not a JSON string.
pub fn redact(
    object: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<CanonicalJsonObject, Error> {
    let mut event = object.clone();

    let event_type_value = match event.get("type") {
        Some(event_type_value) => event_type_value,
        None => return Err(Error::new("field `type` in JSON value must be present")),
    };

    let allowed_content_keys = match event_type_value {
        CanonicalJsonValue::String(event_type) => allowed_content_keys_for(event_type, version),
        _ => return Err(Error::new("field `type` in JSON value must be a JSON string")),
    };

    if let Some(content_value) = event.get_mut("content") {
        let content = match content_value {
            CanonicalJsonValue::Object(map) => map,
            _ => return Err(Error::new("field `content` in JSON value must be a JSON object")),
        };

        let mut old_content = mem::replace(content, BTreeMap::new());

        for &key in allowed_content_keys {
            if let Some(value) = old_content.remove(key) {
                content.insert(key.to_owned(), value);
            }
        }
    }

    let mut old_event = mem::replace(&mut event, BTreeMap::new());

    for &key in ALLOWED_KEYS {
        if let Some(value) = old_event.remove(key) {
            event.insert(key.to_owned(), value);
        }
    }

    Ok(event)
}

#[cfg(test)]
mod tests {
    use ruma_serde::CanonicalJsonValue;
    use serde_json::json;
    use std::convert::TryFrom;

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

        let object = match CanonicalJsonValue::try_from(data).unwrap() {
            CanonicalJsonValue::Object(obj) => obj,
            _ => unreachable!(),
        };

        assert_eq!(canonical_json(&object), canonical);
    }
}
