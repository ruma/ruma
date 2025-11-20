//! Functions for signing and verifying JSON and events.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    mem,
};

use base64::{Engine, alphabet};
use ruma_common::{
    AnyKeyName, CanonicalJsonObject, CanonicalJsonValue, OwnedEventId, OwnedServerName,
    SigningKeyAlgorithm, SigningKeyId, UserId,
    canonical_json::{JsonType, redact},
    room_version_rules::{EventIdFormatVersion, RedactionRules, RoomVersionRules, SignaturesRules},
    serde::{Base64, base64::Standard},
};
use serde_json::to_string as to_json_string;
use sha2::{Sha256, digest::Digest};

#[cfg(test)]
mod tests;

use crate::{
    Error, JsonError, ParseError, VerificationError,
    keys::{KeyPair, PublicKeyMap},
    verification::{Verified, Verifier, verifier_from_algorithm},
};

/// The [maximum size allowed] for a PDU.
///
/// [maximum size allowed]: https://spec.matrix.org/latest/client-server-api/#size-limits
const MAX_PDU_BYTES: usize = 65_535;

/// The fields to remove from a JSON object when converting JSON into the "canonical" form.
static CANONICAL_JSON_FIELDS_TO_REMOVE: &[&str] = &["signatures", "unsigned"];

/// The fields to remove from a JSON object when creating a content hash of an event.
static CONTENT_HASH_FIELDS_TO_REMOVE: &[&str] = &["hashes", "signatures", "unsigned"];

/// The fields to remove from a JSON object when creating a reference hash of an event.
static REFERENCE_HASH_FIELDS_TO_REMOVE: &[&str] = &["signatures", "unsigned"];

/// Signs an arbitrary JSON object and adds the signature to an object under the key `signatures`.
///
/// If `signatures` is already present, the new signature will be appended to the existing ones.
///
/// # Parameters
///
/// * `entity_id`: The identifier of the entity creating the signature. Generally this means a
///   homeserver, e.g. `example.com`.
/// * `key_pair`: A cryptographic key pair used to sign the JSON.
/// * `object`: A JSON object to sign according and append a signature to.
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
/// # use ruma_common::serde::base64::Base64;
/// #
/// const PKCS8: &str = "\
///     MFECAQEwBQYDK2VwBCIEINjozvdfbsGEt6DD+7Uf4PiJ/YvTNXV2mIPc/\
///     tA0T+6tgSEA3TPraTczVkDPTRaX4K+AfUuyx7Mzq1UafTXypnl0t2k\
/// ";
///
/// let document: Base64 = Base64::parse(PKCS8).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = ruma_signatures::Ed25519KeyPair::from_der(
///     document.as_bytes(),
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

    let CanonicalJsonValue::Object(signature_set) = signature_set else {
        return Err(JsonError::not_multiples_of_type("signatures", JsonType::Object));
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
/// [canonical]: https://spec.matrix.org/latest/appendices/#canonical-json
///
/// # Parameters
///
/// * `object`: The JSON object to convert.
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
/// Signatures using an unsupported algorithm are ignored, but each entity must have at least one
/// signature from a supported algorithm.
///
/// Unlike `content_hash` and `reference_hash`, this function does not report an error if the
/// canonical JSON is larger than 65535 bytes; this function may be used for requests that are
/// larger than just one PDU's maximum size.
///
/// # Parameters
///
/// * `public_key_map`: A map from entity identifiers to a map from key identifiers to public keys.
///   Generally, entity identifiers are server names — the host/IP/port of a homeserver (e.g.
///   `example.com`) for which a signature must be verified. Key identifiers for each server (e.g.
///   `ed25519:1`) then map to their respective public keys.
/// * `object`: The JSON object that was signed.
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
        Some(CanonicalJsonValue::Object(signatures)) => signatures,
        Some(_) => return Err(JsonError::not_of_type("signatures", JsonType::Object)),
        None => return Err(JsonError::field_missing_from_object("signatures")),
    };

    let canonical_json = canonical_json(object)?;

    for entity_id in signature_map.keys() {
        verify_canonical_json_for_entity(
            entity_id,
            public_key_map,
            signature_map,
            canonical_json.as_bytes(),
        )?;
    }

    Ok(())
}

/// Uses a set of public keys to verify signed canonical JSON bytes for a given entity.
///
/// Implements the algorithm described in the spec for [checking signatures].
///
/// # Parameters
///
/// * `entity_id`: The entity to check the signatures for.
/// * `public_key_map`: A map from entity identifiers to a map from key identifiers to public keys.
/// * `signature_map`: The map of signatures from the signed JSON object.
/// * `canonical_json`: The signed canonical JSON bytes. Can be obtained by calling
///   [`canonical_json()`].
///
/// # Errors
///
/// Returns an error if verification fails.
///
/// [checking signatures]: https://spec.matrix.org/latest/appendices/#checking-for-a-signature
fn verify_canonical_json_for_entity(
    entity_id: &str,
    public_key_map: &PublicKeyMap,
    signature_map: &CanonicalJsonObject,
    canonical_json: &[u8],
) -> Result<(), Error> {
    let signature_set = match signature_map.get(entity_id) {
        Some(CanonicalJsonValue::Object(set)) => set,
        Some(_) => {
            return Err(JsonError::not_multiples_of_type("signature sets", JsonType::Object));
        }
        None => return Err(VerificationError::NoSignaturesForEntity(entity_id.to_owned()).into()),
    };

    let public_keys = public_key_map
        .get(entity_id)
        .ok_or_else(|| VerificationError::NoPublicKeysForEntity(entity_id.to_owned()))?;

    let mut checked = false;
    for (key_id, signature) in signature_set {
        // If we cannot parse the key ID, ignore.
        let Ok(parsed_key_id) = <&SigningKeyId<AnyKeyName>>::try_from(key_id.as_str()) else {
            continue;
        };

        // If the signature uses an unknown algorithm, ignore.
        let Some(verifier) = verifier_from_algorithm(&parsed_key_id.algorithm()) else {
            continue;
        };

        let Some(public_key) = public_keys.get(key_id) else {
            return Err(VerificationError::PublicKeyNotFound {
                entity: entity_id.to_owned(),
                key_id: key_id.clone(),
            }
            .into());
        };

        let CanonicalJsonValue::String(signature) = signature else {
            return Err(JsonError::not_of_type("signature", JsonType::String));
        };

        let signature = Base64::<Standard>::parse(signature)
            .map_err(|e| ParseError::base64("signature", signature, e))?;

        verify_canonical_json_with(
            &verifier,
            public_key.as_bytes(),
            signature.as_bytes(),
            canonical_json,
        )?;
        checked = true;
    }

    if !checked {
        return Err(VerificationError::NoSupportedSignatureForEntity(entity_id.to_owned()).into());
    }

    Ok(())
}

/// Check a signed JSON object using the given public key and signature, all provided as bytes.
///
/// This is a low-level function. In general you will want to use [`verify_event()`] or
/// [`verify_json()`].
///
/// # Parameters
///
/// * `algorithm`: The algorithm used for the signature. Currently this method only supports the
///   ed25519 algorithm.
/// * `public_key`: The raw bytes of the public key used to sign the JSON.
/// * `signature`: The raw bytes of the signature.
/// * `canonical_json`: The signed canonical JSON bytes. Can be obtained by calling
///   [`canonical_json()`].
///
/// # Errors
///
/// Returns an error if verification fails.
pub fn verify_canonical_json_bytes(
    algorithm: &SigningKeyAlgorithm,
    public_key: &[u8],
    signature: &[u8],
    canonical_json: &[u8],
) -> Result<(), Error> {
    let verifier =
        verifier_from_algorithm(algorithm).ok_or(VerificationError::UnsupportedAlgorithm)?;

    verify_canonical_json_with(&verifier, public_key, signature, canonical_json)
}

/// Uses a public key to verify signed canonical JSON bytes.
///
/// # Parameters
///
/// * `verifier`: A [`Verifier`] appropriate for the digital signature algorithm that was used.
/// * `public_key`: The raw bytes of the public key used to sign the JSON.
/// * `signature`: The raw bytes of the signature.
/// * `canonical_json`: The signed canonical JSON bytes. Can be obtained by calling
///   [`canonical_json()`].
///
/// # Errors
///
/// Returns an error if verification fails.
fn verify_canonical_json_with<V>(
    verifier: &V,
    public_key: &[u8],
    signature: &[u8],
    canonical_json: &[u8],
) -> Result<(), Error>
where
    V: Verifier,
{
    verifier.verify_json(public_key, signature, canonical_json)
}

/// Creates a *content hash* for an event.
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
/// The reference hash of an event covers the essential fields of an event, including content
/// hashes.
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
pub fn reference_hash(
    object: &CanonicalJsonObject,
    rules: &RoomVersionRules,
) -> Result<String, Error> {
    let redacted_value = redact(object.clone(), &rules.redaction, None)?;

    let json =
        canonical_json_with_fields_to_remove(&redacted_value, REFERENCE_HASH_FIELDS_TO_REMOVE)?;
    if json.len() > MAX_PDU_BYTES {
        return Err(Error::PduSize);
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

/// Hashes and signs an event and adds the hash and signature to objects under the keys `hashes` and
/// `signatures`, respectively.
///
/// If `hashes` and/or `signatures` are already present, the new data will be appended to the
/// existing data.
///
/// # Parameters
///
/// * `entity_id`: The identifier of the entity creating the signature. Generally this means a
///   homeserver, e.g. "example.com".
/// * `key_pair`: A cryptographic key pair used to sign the event.
/// * `object`: A JSON object to be hashed and signed according to the Matrix specification.
/// * `redaction_rules`: The redaction rules for the version of the event's room.
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
/// # use ruma_common::{RoomVersionId, serde::base64::Base64};
/// # use ruma_signatures::{hash_and_sign_event, Ed25519KeyPair};
/// #
/// const PKCS8: &str = "\
///     MFECAQEwBQYDK2VwBCIEINjozvdfbsGEt6DD+7Uf4PiJ/YvTNXV2mIPc/\
///     tA0T+6tgSEA3TPraTczVkDPTRaX4K+AfUuyx7Mzq1UafTXypnl0t2k\
/// ";
///
/// let document: Base64 = Base64::parse(PKCS8).unwrap();
///
/// // Create an Ed25519 key pair.
/// let key_pair = Ed25519KeyPair::from_der(
///     document.as_bytes(),
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
/// // Get the rules for the version of the current room.
/// let rules =
///     RoomVersionId::V1.rules().expect("The rules should be known for a supported room version");
///
/// // Hash and sign the JSON with the key pair.
/// assert!(hash_and_sign_event("domain", &key_pair, &mut object, &rules.redaction).is_ok());
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
    redaction_rules: &RedactionRules,
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

    let mut redacted = redact(object.clone(), redaction_rules, None)?;

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
/// If the `Ok` variant is returned by this function, it will contain a [`Verified`] value which
/// distinguishes an event with valid signatures and a matching content hash with an event with
/// only valid signatures. See the documentation for [`Verified`] for details.
///
/// # Parameters
///
/// * `public_key_map`: A map from entity identifiers to a map from key identifiers to public keys.
///   Generally, entity identifiers are server names—the host/IP/port of a homeserver (e.g.
///   "example.com") for which a signature must be verified. Key identifiers for each server (e.g.
///   "ed25519:1") then map to their respective public keys.
/// * `object`: The JSON object of the event that was signed.
/// * `room_version`: The version of the event's room.
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
/// // Get the redaction rules for the version of the current room.
/// let rules =
///     RoomVersionId::V6.rules().expect("The rules should be known for a supported room version");
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// let verification_result = verify_event(&public_key_map, &object, &rules);
/// assert!(verification_result.is_ok());
/// assert_eq!(verification_result.unwrap(), Verified::All);
/// ```
pub fn verify_event(
    public_key_map: &PublicKeyMap,
    object: &CanonicalJsonObject,
    rules: &RoomVersionRules,
) -> Result<Verified, Error> {
    let redacted = redact(object.clone(), &rules.redaction, None)?;

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

    let servers_to_check = servers_to_check_signatures(object, &rules.signatures)?;
    let canonical_json = canonical_json(&redacted)?;

    for entity_id in servers_to_check {
        verify_canonical_json_for_entity(
            entity_id.as_str(),
            public_key_map,
            signature_map,
            canonical_json.as_bytes(),
        )?;
    }

    let calculated_hash = content_hash(object)?;

    if let Ok(hash) = Base64::<Standard>::parse(hash)
        && hash.as_bytes() == calculated_hash.as_bytes()
    {
        return Ok(Verified::All);
    }

    Ok(Verified::Signatures)
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

/// Extracts the server names to check signatures for given event.
///
/// Respects the rules for [validating signatures on received events] for populating the result:
///
/// - Add the server of the sender, except if it's an invite event that results from a third-party
///   invite.
/// - For room versions 1 and 2, add the server of the `event_id`.
/// - For room versions that support restricted join rules, if it's a join event with a
///   `join_authorised_via_users_server`, add the server of that user.
///
/// [validating signatures on received events]: https://spec.matrix.org/latest/server-server-api/#validating-hashes-and-signatures-on-received-events
fn servers_to_check_signatures(
    object: &CanonicalJsonObject,
    rules: &SignaturesRules,
) -> Result<BTreeSet<OwnedServerName>, Error> {
    let mut servers_to_check = BTreeSet::new();

    if !is_invite_via_third_party_id(object)? {
        match object.get("sender") {
            Some(CanonicalJsonValue::String(raw_sender)) => {
                let user_id = <&UserId>::try_from(raw_sender.as_str())
                    .map_err(|e| Error::from(ParseError::UserId(e)))?;

                servers_to_check.insert(user_id.server_name().to_owned());
            }
            Some(_) => return Err(JsonError::not_of_type("sender", JsonType::String)),
            _ => return Err(JsonError::field_missing_from_object("sender")),
        }
    }

    if rules.check_event_id_server {
        match object.get("event_id") {
            Some(CanonicalJsonValue::String(raw_event_id)) => {
                let event_id: OwnedEventId =
                    raw_event_id.parse().map_err(|e| Error::from(ParseError::EventId(e)))?;

                let server_name = event_id
                    .server_name()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| ParseError::server_name_from_event_id(event_id))?;

                servers_to_check.insert(server_name);
            }
            Some(_) => return Err(JsonError::not_of_type("event_id", JsonType::String)),
            _ => {
                return Err(JsonError::field_missing_from_object("event_id"));
            }
        }
    }

    if rules.check_join_authorised_via_users_server
        && let Some(authorized_user) = object
            .get("content")
            .and_then(|c| c.as_object())
            .and_then(|c| c.get("join_authorised_via_users_server"))
    {
        let authorized_user = authorized_user.as_str().ok_or_else(|| {
            JsonError::not_of_type("join_authorised_via_users_server", JsonType::String)
        })?;
        let authorized_user =
            <&UserId>::try_from(authorized_user).map_err(|e| Error::from(ParseError::UserId(e)))?;

        servers_to_check.insert(authorized_user.server_name().to_owned());
    }

    Ok(servers_to_check)
}

/// Whether the given event is an `m.room.member` invite that was created as the result of a
/// third-party invite.
///
/// Returns an error if the object has not the expected format of an `m.room.member` event.
fn is_invite_via_third_party_id(object: &CanonicalJsonObject) -> Result<bool, Error> {
    let Some(CanonicalJsonValue::String(raw_type)) = object.get("type") else {
        return Err(JsonError::not_of_type("type", JsonType::String));
    };

    if raw_type != "m.room.member" {
        return Ok(false);
    }

    let Some(CanonicalJsonValue::Object(content)) = object.get("content") else {
        return Err(JsonError::not_of_type("content", JsonType::Object));
    };

    let Some(CanonicalJsonValue::String(membership)) = content.get("membership") else {
        return Err(JsonError::not_of_type("membership", JsonType::String));
    };

    if membership != "invite" {
        return Ok(false);
    }

    match content.get("third_party_invite") {
        Some(CanonicalJsonValue::Object(_)) => Ok(true),
        None => Ok(false),
        _ => Err(JsonError::not_of_type("third_party_invite", JsonType::Object)),
    }
}
