//! Functions for signing and verifying JSON and events.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    mem,
};

use base64::{encode_config, STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use ruma_common::{
    serde::{base64::Standard, Base64, CanonicalJsonObject, CanonicalJsonValue},
    OwnedEventId, OwnedServerName, RoomVersionId, UserId,
};
use serde_json::{from_str as from_json_str, to_string as to_json_string};
use sha2::{digest::Digest, Sha256};

use crate::{
    keys::{KeyPair, PublicKeyMap},
    split_id,
    verification::{Ed25519Verifier, Verified, Verifier},
    Error, JsonError, JsonType, ParseError, VerificationError,
};

const MAX_PDU_BYTES: usize = 65_535;

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
        "m.room.member" => match version {
            RoomVersionId::V9 => &["membership", "join_authorised_via_users_server"],
            _ => &["membership"],
        },
        "m.room.create" => &["creator"],
        "m.room.join_rules" => match version {
            RoomVersionId::V8 | RoomVersionId::V9 => &["join_rule", "allow"],
            _ => &["join_rule"],
        },
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
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5 => &["aliases"],
            // All other room versions, including custom ones, are treated by version 6 rules.
            // TODO: Should we return an error for unknown versions instead?
            _ => &[],
        },
        #[cfg(feature = "unstable-msc2870")]
        "m.room.server_acl" if version.as_str() == "org.matrix.msc2870" => {
            &["allow", "deny", "allow_ip_literals"]
        }
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
/// )
/// .unwrap();
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
    let json = to_json_string(object).map_err(JsonError::Serde)?;

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
/// [canonical]: https://spec.matrix.org/v1.2/appendices/#canonical-json
///
/// # Parameters
///
/// * object: The JSON object to convert.
///
/// # Examples
///
/// ```rust
/// let input = r#"{
///     "本": 2,
///     "日": 1
/// }"#;
///
/// let object = serde_json::from_str(input).unwrap();
/// let canonical = ruma_signatures::canonical_json(&object).unwrap();
///
/// assert_eq!(canonical, r#"{"日":1,"本":2}"#);
/// ```
pub fn canonical_json(object: &CanonicalJsonObject) -> Result<String, Error> {
    canonical_json_with_fields_to_remove(object, CANONICAL_JSON_FIELDS_TO_REMOVE)
}

/// Uses a set of public keys to verify a signed JSON object.
///
/// Unlike `content_hash` and `reference_hash`, this function does not report an error if the
/// canonical JSON is larger than 65535 bytes; this function may be used for requests that are
/// larger than just one PDU's maximum size.
///
/// # Parameters
///
/// * public_key_map: A map from entity identifiers to a map from key identifiers to public keys.
/// Generally, entity identifiers are server names — the host/IP/port of a homeserver (e.g.
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
/// use ruma_common::serde::Base64;
///
/// const PUBLIC_KEY: &[u8] = b"XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
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
/// public_key_set.insert("ed25519:1".into(), Base64::parse(PUBLIC_KEY.to_owned()).unwrap());
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

            let signature = Base64::<Standard>::parse(signature)
                .map_err(|e| ParseError::base64("signature", signature, e))?;

            verify_json_with(
                &Ed25519Verifier,
                public_key.as_bytes(),
                signature.as_bytes(),
                object,
            )?;
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
    verifier.verify_json(public_key, signature, canonical_json(object)?.as_bytes())
}

/// Creates a *content hash* for an event.
///
/// The content hash of an event covers the complete event including the unredacted contents. It is
/// used during federation and is described in the Matrix server-server specification.
///
/// # Parameters
///
/// object: A JSON object to generate a content hash for.
///
/// # Errors
///
/// Returns an error if the event is too large.
pub fn content_hash(object: &CanonicalJsonObject) -> Result<Base64<Standard, [u8; 32]>, Error> {
    let json = canonical_json_with_fields_to_remove(object, CONTENT_HASH_FIELDS_TO_REMOVE)?;
    if json.len() > MAX_PDU_BYTES {
        return Err(Error::PduSize);
    }

    let hash = Sha256::digest(json.as_bytes());

    Ok(Base64::new(hash.into()))
}

/// Creates a *reference hash* for an event.
///
/// Returns the hash as a base64-encoded string, using the standard character set, without padding.
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
/// Returns an error if the event is too large or redaction fails.
pub fn reference_hash(
    value: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<String, Error> {
    let redacted_value = redact(value, version)?;

    let json =
        canonical_json_with_fields_to_remove(&redacted_value, REFERENCE_HASH_FIELDS_TO_REMOVE)?;
    if json.len() > MAX_PDU_BYTES {
        return Err(Error::PduSize);
    }

    let hash = Sha256::digest(json.as_bytes());

    Ok(encode_config(
        &hash,
        match version {
            RoomVersionId::V1 | RoomVersionId::V2 | RoomVersionId::V3 => STANDARD_NO_PAD,
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
/// # use ruma_common::RoomVersionId;
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
/// )
/// .unwrap();
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
///     }"#,
/// )
/// .unwrap();
///
/// // Hash and sign the JSON with the key pair.
/// assert!(hash_and_sign_event("domain", &key_pair, &mut object, &RoomVersionId::V1).is_ok());
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
    let hash = content_hash(object)?;

    let hashes_value = object
        .entry("hashes".to_owned())
        .or_insert_with(|| CanonicalJsonValue::Object(BTreeMap::new()));

    match hashes_value {
        CanonicalJsonValue::Object(hashes) => {
            hashes.insert("sha256".into(), CanonicalJsonValue::String(hash.encode()))
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
/// # use ruma_common::RoomVersionId;
/// # use ruma_common::serde::Base64;
/// # use ruma_signatures::{verify_event, Verified};
/// #
/// const PUBLIC_KEY: &[u8] = b"XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
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
/// public_key_set.insert("ed25519:1".into(), Base64::parse(PUBLIC_KEY.to_owned()).unwrap());
/// let mut public_key_map = BTreeMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// let verification_result = verify_event(&public_key_map, &object, &RoomVersionId::V6);
/// assert!(verification_result.is_ok());
/// assert_eq!(verification_result.unwrap(), Verified::All);
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
    let canonical_json = from_json_str(&canonical_json(&redacted)?).map_err(JsonError::from)?;

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

        let signature = Base64::<Standard>::parse(signature)
            .map_err(|e| ParseError::base64("signature", signature, e))?;

        verify_json_with(
            &Ed25519Verifier,
            public_key.as_bytes(),
            signature.as_bytes(),
            &canonical_json,
        )?;
    }

    let calculated_hash = content_hash(object)?;

    if let Ok(hash) = Base64::<Standard>::parse(hash) {
        if hash.as_bytes() == calculated_hash.as_bytes() {
            return Ok(Verified::All);
        }
    }

    Ok(Verified::Signatures)
}

struct SignatureAndPubkey<'a> {
    signature: &'a CanonicalJsonValue,
    public_key: &'a Base64,
}

/// Internal implementation detail of the canonical JSON algorithm.
///
/// Allows customization of the fields that will be removed before serializing.
fn canonical_json_with_fields_to_remove(
    object: &CanonicalJsonObject,
    fields: &[&str],
) -> Result<String, Error> {
    let mut owned_object = object.clone();

    for field in fields {
        owned_object.remove(*field);
    }

    to_json_string(&owned_object).map_err(|e| Error::Json(e.into()))
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
    let mut val = object.clone();
    redact_in_place(&mut val, version)?;
    Ok(val)
}

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// Functionally equivalent to `redact`, only;
/// * upon error, the event is not touched.
/// * this'll redact the event in-place.
pub fn redact_in_place(
    event: &mut CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<(), Error> {
    // Get the content keys here instead of the event type, because we cant teach rust that this is
    // a disjoint borrow.
    let allowed_content_keys: &[&str] = match event.get("type") {
        Some(CanonicalJsonValue::String(event_type)) => {
            allowed_content_keys_for(event_type, version)
        }
        Some(_) => return Err(JsonError::not_of_type("type", JsonType::String)),
        None => return Err(JsonError::field_missing_from_object("type")),
    };

    if let Some(content_value) = event.get_mut("content") {
        let content = match content_value {
            CanonicalJsonValue::Object(map) => map,
            _ => return Err(JsonError::not_of_type("content", JsonType::Object)),
        };

        object_retain_keys(content, allowed_content_keys);
    }

    let mut old_event = mem::take(event);

    for &key in ALLOWED_KEYS {
        if let Some(value) = old_event.remove(key) {
            event.insert(key.to_owned(), value);
        }
    }

    Ok(())
}

/// Redacts event content using the rules specified in the Matrix client-server specification.
///
/// Edits the `object` in-place.
pub fn redact_content_in_place(
    object: &mut CanonicalJsonObject,
    version: &RoomVersionId,
    event_type: impl AsRef<str>,
) {
    object_retain_keys(object, allowed_content_keys_for(event_type.as_ref(), version))
}

fn object_retain_keys(object: &mut CanonicalJsonObject, keys: &[&str]) {
    let mut old_content = mem::take(object);

    for &key in keys {
        if let Some(value) = old_content.remove(key) {
            object.insert(key.to_owned(), value);
        }
    }
}

/// Extracts the server names to check signatures for given event.
///
/// It will return the sender's server (unless it's a third party invite) and the event id server
/// (on v1 and v2 room versions)
///
/// Starting with room version 8, if join_authorised_via_users_server is present, a signature from
/// that user is required.
fn servers_to_check_signatures(
    object: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<BTreeSet<OwnedServerName>, Error> {
    let mut servers_to_check = BTreeSet::new();

    if !is_third_party_invite(object)? {
        match object.get("sender") {
            Some(CanonicalJsonValue::String(raw_sender)) => {
                let user_id = <&UserId>::try_from(raw_sender.as_str())
                    .map_err(|e| Error::from(ParseError::UserId(e)))?;

                servers_to_check.insert(user_id.server_name().to_owned());
            }
            _ => return Err(JsonError::not_of_type("sender", JsonType::String)),
        };
    }

    match version {
        RoomVersionId::V1 | RoomVersionId::V2 => match object.get("event_id") {
            Some(CanonicalJsonValue::String(raw_event_id)) => {
                let event_id: OwnedEventId =
                    raw_event_id.parse().map_err(|e| Error::from(ParseError::EventId(e)))?;

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
        RoomVersionId::V3
        | RoomVersionId::V4
        | RoomVersionId::V5
        | RoomVersionId::V6
        | RoomVersionId::V7 => {}
        // TODO: And for all future versions that have join_authorised_via_users_server
        RoomVersionId::V8 | RoomVersionId::V9 => {
            if let Some(authorized_user) = object
                .get("content")
                .and_then(|c| c.as_object())
                .and_then(|c| c.get("join_authorised_via_users_server"))
            {
                let authorized_user = authorized_user.as_str().ok_or_else(|| {
                    JsonError::not_of_type("join_authorised_via_users_server", JsonType::String)
                })?;
                let authorized_user = <&UserId>::try_from(authorized_user)
                    .map_err(|e| Error::from(ParseError::UserId(e)))?;

                servers_to_check.insert(authorized_user.server_name().to_owned());
            }
        }
        _ => unimplemented!(),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_common::{
        serde::{Base64, CanonicalJsonValue},
        RoomVersionId, ServerSigningKeyId, SigningKeyAlgorithm,
    };
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

        assert_eq!(canonical_json(&object).unwrap(), canonical);
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
        let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionId::V6);

        assert!(verification_result.is_ok());
        let verification = verification_result.unwrap();
        assert!(matches!(verification, Verified::Signatures));
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

        let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionId::V1);

        assert!(verification_result.is_ok());
        let verification = verification_result.unwrap();
        assert!(matches!(verification, Verified::Signatures));
    }

    #[test]
    fn verify_event_check_signatures_for_authorized_user() {
        let key_pair_sender = generate_key_pair();
        let key_pair_authorized = generate_key_pair();
        let mut signed_event = serde_json::from_str(
            r#"{
                "event_id": "$event_id:domain-event",
                "auth_events": [],
                "content": {"join_authorised_via_users_server": "@authorized:domain-authorized"},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "m.room.member",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
        )
        .unwrap();
        sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();
        sign_json("domain-authorized", &key_pair_authorized, &mut signed_event).unwrap();

        let mut public_key_map = BTreeMap::new();
        add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);
        add_key_to_map(&mut public_key_map, "domain-authorized", &key_pair_authorized);

        let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionId::V9);

        assert!(verification_result.is_ok());
        let verification = verification_result.unwrap();
        assert!(matches!(verification, Verified::Signatures));
    }

    #[test]
    fn verification_fails_if_missing_signatures_for_authorized_user() {
        let key_pair_sender = generate_key_pair();
        let mut signed_event = serde_json::from_str(
            r#"{
                "event_id": "$event_id:domain-event",
                "auth_events": [],
                "content": {"join_authorised_via_users_server": "@authorized:domain-authorized"},
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

        let mut public_key_map = BTreeMap::new();
        add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);

        let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionId::V9);

        assert!(verification_result.is_err()) // Should be Err(VerificationError::
                                              // signature_not_found("domain-authorized")));
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
        let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionId::V6);

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
        let encoded_public_key = Base64::new(newly_generated_key_pair.public_key().to_owned());
        let version = ServerSigningKeyId::from_parts(
            SigningKeyAlgorithm::Ed25519,
            key_pair_sender.version().try_into().unwrap(),
        );
        sender_key_map.insert(version.to_string(), encoded_public_key);
        public_key_map.insert("domain-sender".to_owned(), sender_key_map);

        let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionId::V6);

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
        let encoded_public_key = Base64::new(pair.public_key().to_owned());
        let version = ServerSigningKeyId::from_parts(
            SigningKeyAlgorithm::Ed25519,
            pair.version().try_into().unwrap(),
        );

        sender_key_map.insert(version.to_string(), encoded_public_key);

        public_key_map.insert(name.to_owned(), sender_key_map);
    }
}
