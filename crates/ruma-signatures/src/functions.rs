//! Functions for signing and verifying JSON and events.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    mem,
    str::FromStr,
};

use base64::{decode_config, encode_config, Config, STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use ed25519_dalek::Digest;
use ruma_identifiers::{EventId, RoomVersionId, ServerNameBox, UserId};
use ruma_serde::{to_canonical_json_string, CanonicalJsonObject, CanonicalJsonValue};
use serde_json::from_str as from_json_str;
use sha2::Sha256;

use crate::{
    keys::{KeyPair, PublicKeyMap},
    split_id,
    verification::{Ed25519Verifier, Verified, Verifier},
    Error, JsonError, JsonType, ParseError, VerificationError,
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
            | RoomVersionId::Version5 => &["aliases"],
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
/// let key_pair = ruma_signatures::Ed25519KeyPair::from_der(
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
    let (signatures_key, mut signature_map) = match object.remove_entry("signatures") {
        Some((key, CanonicalJsonValue::Object(signatures))) => (Cow::Owned(key), signatures),
        Some(_) => return Err(JsonError::not_of_type("signatures", JsonType::Object)),
        None => (Cow::Borrowed("signatures"), BTreeMap::new()),
    };

    let maybe_unsigned_entry = object.remove_entry("unsigned");

    // Get the canonical JSON string.
    let json = to_canonical_json_string(object).map_err(JsonError::CanonicalJson)?;

    // Sign the canonical JSON string.
    let signature = key_pair.sign(json.as_bytes());

    // Insert the new signature in the map we pulled out (or created) previously.
    let signature_set = signature_map
        .entry(entity_id.to_owned())
        .or_insert_with(|| CanonicalJsonValue::Object(BTreeMap::new()));

    let signature_set = match signature_set {
        CanonicalJsonValue::Object(obj) => obj,
        _ => return Err(JsonError::not_multiples_of_type("signatures", JsonType::Object)),
    };

    signature_set.insert(signature.id(), CanonicalJsonValue::String(signature.base64()));

    // Put `signatures` and `unsigned` back in.
    object.insert(signatures_key.into(), CanonicalJsonValue::Object(signature_map));

    if let Some((k, v)) = maybe_unsigned_entry {
        object.insert(k, v);
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
/// public_key_set.insert("ed25519:1".into(), PUBLIC_KEY.to_owned());
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
        Some(_) => return Err(JsonError::not_of_type("signatures", JsonType::Object)),
        None => return Err(JsonError::field_missing_from_object("signatures")),
    };

    for (entity_id, signature_set) in signature_map {
        let signature_set = match signature_set {
            CanonicalJsonValue::Object(set) => set,
            _ => return Err(JsonError::not_multiples_of_type("signature sets", JsonType::Object)),
        };

        let public_keys = match public_key_map.get(&entity_id) {
            Some(keys) => keys,
            None => {
                return Err(JsonError::key_missing("public_key_map", "public_keys", &entity_id))
            }
        };

        for (key_id, signature) in &signature_set {
            let signature = match signature {
                CanonicalJsonValue::String(s) => s,
                _ => return Err(JsonError::not_of_type("signature", JsonType::String)),
            };

            let public_key = public_keys.get(key_id).ok_or_else(|| {
                JsonError::key_missing(
                    format!("public_keys of {}", &entity_id),
                    "signature",
                    key_id,
                )
            })?;

            let verify = |config: Config| {
                let signature_bytes = decode_config(signature, config)
                    .map_err(|e| ParseError::base64("signature", signature, e))?;

                let public_key_bytes = decode_config(&public_key, config)
                    .map_err(|e| ParseError::base64("public key", public_key, e))?;

                verify_json_with(&Ed25519Verifier, &public_key_bytes, &signature_bytes, object)
            };

            #[cfg(feature = "compat")]
            also_try_forgiving_base64(STANDARD_NO_PAD, verify)?;
            #[cfg(not(feature = "compat"))]
            verify(STANDARD_NO_PAD)?;
        }
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
    let hash = Sha256::digest(json.as_bytes());

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

    let hash = Sha256::digest(json.as_bytes());

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
/// let key_pair = Ed25519KeyPair::from_der(
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
        _ => return Err(JsonError::not_of_type("hashes", JsonType::Object)),
    };

    let mut redacted = redact(object, version)?;

    sign_json(entity_id, key_pair, &mut redacted)?;

    object.insert("signatures".into(), mem::take(redacted.get_mut("signatures").unwrap()));

    Ok(())
}

/// Verifies that the signed event contains all the required valid signatures.
///
/// Some room versions may require signatures from multiple homeservers, so this function takes a
/// map from servers to sets of public keys. Signatures are verified for each required homeserver.
/// All known public keys for a homeserver should be provided. The first one found on the given
/// event will be used.
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
/// * version: Room version of the given event
///
/// # Examples
///
/// ```rust
/// # use std::collections::BTreeMap;
/// # use ruma_identifiers::RoomVersionId;
/// # use ruma_signatures::verify_event;
/// # use ruma_signatures::Verified;
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
/// public_key_set.insert("ed25519:1".into(), PUBLIC_KEY.to_owned());
/// let mut public_key_map = BTreeMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// let verification_result = verify_event(&public_key_map, &object, &RoomVersionId::Version6);
/// assert!(verification_result.is_ok());
/// assert!(matches!(verification_result.unwrap(), Verified::All));
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
                    _ => return Err(JsonError::not_of_type("sha256 hash", JsonType::String)),
                },
                None => return Err(JsonError::not_of_type("hashes", JsonType::Object)),
            },
            _ => return Err(JsonError::field_missing_from_object("sha256")),
        },
        None => return Err(JsonError::field_missing_from_object("hashes")),
    };

    let signature_map = match object.get("signatures") {
        Some(CanonicalJsonValue::Object(signatures)) => signatures,
        Some(_) => return Err(JsonError::not_of_type("signatures", JsonType::Object)),
        None => return Err(JsonError::field_missing_from_object("signatures")),
    };

    let servers_to_check = servers_to_check_signatures(object, version)?;
    let canonical_json = from_json_str(&canonical_json(&redacted)).map_err(JsonError::from)?;

    for entity_id in servers_to_check {
        let signature_set = match signature_map.get(entity_id.as_str()) {
            Some(CanonicalJsonValue::Object(set)) => set,
            Some(_) => {
                return Err(JsonError::not_multiples_of_type("signature sets", JsonType::Object))
            }
            None => return Err(VerificationError::signature_not_found(entity_id)),
        };

        let mut maybe_signature_and_public_key = None;

        let public_keys = public_key_map
            .get(entity_id.as_str())
            .ok_or_else(|| VerificationError::public_key_not_found(entity_id))?;

        for (key_id, public_key) in public_keys {
            // Since only ed25519 is supported right now, we don't actually need to check what the
            // algorithm is. If it split successfully, it's ed25519.
            if split_id(key_id).is_err() {
                break;
            }

            if let Some(signature) = signature_set.get(key_id) {
                maybe_signature_and_public_key = Some(SignatureAndPubkey { signature, public_key });

                break;
            }
        }

        let signature_and_pubkey = match maybe_signature_and_public_key {
            Some(value) => value,
            None => return Err(VerificationError::UnknownPublicKeysForSignature.into()),
        };

        let signature = match signature_and_pubkey.signature {
            CanonicalJsonValue::String(signature) => signature,
            _ => return Err(JsonError::not_of_type("signature", JsonType::String)),
        };

        let public_key = signature_and_pubkey.public_key;

        let verify = |config: Config| {
            let signature_bytes = decode_config(signature, config)
                .map_err(|e| ParseError::base64("signature", signature, e))?;

            let public_key_bytes = decode_config(&public_key, config)
                .map_err(|e| ParseError::base64("public key", public_key, e))?;

            verify_json_with(&Ed25519Verifier, &public_key_bytes, &signature_bytes, &canonical_json)
        };

        #[cfg(feature = "compat")]
        also_try_forgiving_base64(STANDARD_NO_PAD, verify)?;
        #[cfg(not(feature = "compat"))]
        verify(STANDARD_NO_PAD)?;
    }

    let calculated_hash = content_hash(object);

    if *hash == calculated_hash {
        Ok(Verified::All)
    } else {
        Ok(Verified::Signatures)
    }
}

struct SignatureAndPubkey<'a> {
    signature: &'a CanonicalJsonValue,
    public_key: &'a String,
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
        None => return Err(JsonError::field_missing_from_object("type")),
    };

    let allowed_content_keys = match event_type_value {
        CanonicalJsonValue::String(event_type) => allowed_content_keys_for(event_type, version),
        _ => return Err(JsonError::not_of_type("type", JsonType::String)),
    };

    if let Some(content_value) = event.get_mut("content") {
        let content = match content_value {
            CanonicalJsonValue::Object(map) => map,
            _ => return Err(JsonError::not_of_type("content", JsonType::Object)),
        };

        let mut old_content = mem::take(content);

        for &key in allowed_content_keys {
            if let Some(value) = old_content.remove(key) {
                content.insert(key.to_owned(), value);
            }
        }
    }

    let mut old_event = mem::take(&mut event);

    for &key in ALLOWED_KEYS {
        if let Some(value) = old_event.remove(key) {
            event.insert(key.to_owned(), value);
        }
    }

    Ok(event)
}

/// Extracts the server names to check signatures for given event. It will return the sender's
/// server (unless it's a third party invite) and the event id server (on v1 and v2 room versions)
fn servers_to_check_signatures(
    object: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<BTreeSet<ServerNameBox>, Error> {
    let mut servers_to_check = BTreeSet::new();

    if !is_third_party_invite(object)? {
        match object.get("sender") {
            Some(CanonicalJsonValue::String(raw_sender)) => {
                let user_id =
                    UserId::from_str(raw_sender).map_err(|e| Error::from(ParseError::UserId(e)))?;

                servers_to_check.insert(user_id.server_name().to_owned());
            }
            _ => return Err(JsonError::not_of_type("sender", JsonType::String)),
        };
    }

    match version {
        RoomVersionId::Version1 | RoomVersionId::Version2 => match object.get("event_id") {
            Some(CanonicalJsonValue::String(raw_event_id)) => {
                let event_id = EventId::from_str(raw_event_id)
                    .map_err(|e| Error::from(ParseError::EventId(e)))?;

                let server_name = event_id
                    .server_name()
                    .ok_or_else(|| ParseError::from_event_id_by_room_version(&event_id, version))?
                    .to_owned();

                servers_to_check.insert(server_name);
            }
            _ => {
                return Err(JsonError::field_missing_from_object("event_id"));
            }
        },
        _ => (),
    }

    Ok(servers_to_check)
}

/// Checks if `object` contains an event of type `m.room.third_party_invite`
fn is_third_party_invite(object: &CanonicalJsonObject) -> Result<bool, Error> {
    match object.get("type") {
        Some(CanonicalJsonValue::String(raw_type)) => Ok(raw_type == "m.room.third_party_invite"),
        _ => Err(JsonError::not_of_type("type", JsonType::String)),
    }
}

#[cfg(feature = "compat")]
// see https://github.com/ruma/ruma/issues/591
// synapse allows this, so we must allow it too.
// shouldn't lose data, but when it does, it'll make verification fail instead.
pub(crate) fn also_try_forgiving_base64<T, E>(
    config: Config,
    twice: impl Fn(Config) -> Result<T, E>,
) -> Result<T, E>
where
    E: std::fmt::Display,
{
    use tracing::{debug, warn};

    let first_try = match twice(config) {
        Ok(t) => return Ok(t),
        Err(e) => e,
    };

    let adjusted = config.decode_allow_trailing_bits(true);

    match twice(adjusted) {
        Ok(t) => {
            warn!(
                "Usage of base64 config only worked after allowing trailing bits, first error: {}",
                first_try
            );
            Ok(t)
        }
        Err(e) => {
            debug!("Second error when trying to allow trailing bits: {}", e);
            Err(first_try)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        convert::{TryFrom, TryInto},
    };

    use base64::{encode_config, STANDARD_NO_PAD};
    use ruma_identifiers::{RoomVersionId, ServerSigningKeyId, SigningKeyAlgorithm};
    use ruma_serde::CanonicalJsonValue;
    use serde_json::json;

    use super::canonical_json;
    use crate::{
        sign_json, verify_event, Ed25519KeyPair, Error, PublicKeyMap, PublicKeySet,
        VerificationError, Verified,
    };

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

    #[test]
    fn verify_event_does_not_check_signatures_for_third_party_invites() {
        let signed_event = serde_json::from_str(
            r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@a:domain",
                "signatures": {
                    "domain": {
                        "ed25519:1": "KxwGjPSDEtvnFgU00fwFz+l6d2pJM6XBIaMEn81SXPTRl16AqLAYqfIReFGZlHi5KLjAWbOoMszkwsQma+lYAg"
                    }
                },
                "type": "m.room.third_party_invite",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#
        ).unwrap();

        let public_key_map = BTreeMap::new();
        let verification_result =
            verify_event(&public_key_map, &signed_event, &RoomVersionId::Version6);

        assert!(verification_result.is_ok());
        let verification = verification_result.unwrap();
        assert!(matches!(verification, Verified::Signatures));
    }

    #[cfg(feature = "compat")]
    #[test]
    fn fallback_invalid_base64() {
        use base64::{decode_config, Config};

        const SLIGHTLY_MALFORMED_BASE64: &str = "3UmJnEIzUr2xWyaUnJg5fXwRybwG5FVC6GqMHverEUn0ztuIsvVxX89JXX2pvdTsOBbLQx+4TVL02l4Cp5wPCm";

        let verify = |config: Config| decode_config(SLIGHTLY_MALFORMED_BASE64, config);

        assert!(verify(STANDARD_NO_PAD).is_err());

        assert!(super::also_try_forgiving_base64(STANDARD_NO_PAD, verify).is_ok());
    }

    #[test]
    fn verify_event_check_signatures_for_both_sender_and_event_id() {
        let key_pair_sender = generate_key_pair();
        let key_pair_event = generate_key_pair();
        let mut signed_event = serde_json::from_str(
            r#"{
                "event_id": "$event_id:domain-event",
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
        )
        .unwrap();
        sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();
        sign_json("domain-event", &key_pair_event, &mut signed_event).unwrap();

        let mut public_key_map = BTreeMap::new();
        add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);
        add_key_to_map(&mut public_key_map, "domain-event", &key_pair_event);

        let verification_result =
            verify_event(&public_key_map, &signed_event, &RoomVersionId::Version1);

        assert!(verification_result.is_ok());
        let verification = verification_result.unwrap();
        assert!(matches!(verification, Verified::Signatures));
    }

    #[test]
    fn verification_fails_if_required_keys_are_not_given() {
        let key_pair_sender = generate_key_pair();

        let mut signed_event = serde_json::from_str(
            r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
        )
        .unwrap();
        sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();

        // Verify with an empty public key map should fail due to missing public keys
        let public_key_map = BTreeMap::new();
        let verification_result =
            verify_event(&public_key_map, &signed_event, &RoomVersionId::Version6);

        assert!(verification_result.is_err());
        let error_msg = verification_result.err().unwrap();
        if let Error::Verification(VerificationError::PublicKeyNotFound(entity)) = error_msg {
            assert_eq!(entity, "domain-sender");
        } else {
            panic!("Error was not VerificationError::UnknownPublicKeysForEvent: {:?}", error_msg);
        };
    }

    #[test]
    fn verify_event_fails_if_public_key_is_invalid() {
        let key_pair_sender = generate_key_pair();

        let mut signed_event = serde_json::from_str(
            r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
        )
        .unwrap();
        sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();

        let mut public_key_map = PublicKeyMap::new();
        let mut sender_key_map = PublicKeySet::new();
        let newly_generated_key_pair = generate_key_pair();
        let encoded_public_key =
            encode_config(newly_generated_key_pair.public_key(), STANDARD_NO_PAD);
        let version = ServerSigningKeyId::from_parts(
            SigningKeyAlgorithm::Ed25519,
            key_pair_sender.version().try_into().unwrap(),
        );
        sender_key_map.insert(version.to_string(), encoded_public_key);
        public_key_map.insert("domain-sender".to_owned(), sender_key_map);

        let verification_result =
            verify_event(&public_key_map, &signed_event, &RoomVersionId::Version6);

        assert!(verification_result.is_err());
        let error_msg = verification_result.err().unwrap();
        if let Error::Verification(VerificationError::Signature(error)) = error_msg {
            // dalek doesn't expose InternalError :(
            // https://github.com/dalek-cryptography/ed25519-dalek/issues/174
            assert!(format!("{:?}", error).contains("Some(Verification equation was not satisfied)"))
        } else {
            panic!("Error was not VerificationError::Signature: {:?}", error_msg);
        };
    }

    fn generate_key_pair() -> Ed25519KeyPair {
        let key_content = Ed25519KeyPair::generate().unwrap();
        Ed25519KeyPair::from_der(&key_content, "1".to_owned())
            .unwrap_or_else(|_| panic!("{:?}", &key_content))
    }

    fn add_key_to_map(public_key_map: &mut PublicKeyMap, name: &str, pair: &Ed25519KeyPair) {
        let mut sender_key_map = PublicKeySet::new();
        let encoded_public_key = encode_config(pair.public_key(), STANDARD_NO_PAD);
        let version = ServerSigningKeyId::from_parts(
            SigningKeyAlgorithm::Ed25519,
            pair.version().try_into().unwrap(),
        );

        sender_key_map.insert(version.to_string(), encoded_public_key);

        public_key_map.insert(name.to_owned(), sender_key_map);
    }
}
