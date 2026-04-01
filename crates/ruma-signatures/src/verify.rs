//! Verification of digital signatures.

use std::collections::{BTreeMap, BTreeSet};

use ruma_common::{
    AnyKeyName, CanonicalJsonObject, CanonicalJsonValue, IdParseError, OwnedEventId,
    OwnedServerName, SigningKeyAlgorithm, SigningKeyId, UserId,
    canonical_json::{CanonicalJsonFieldError, CanonicalJsonObjectExt, CanonicalJsonType, redact},
    room_version_rules::{RoomVersionRules, SignaturesRules},
    serde::{Base64, base64::Standard},
};
use serde_json::to_string as to_json_string;

#[cfg(test)]
mod tests;

use crate::{
    JsonError, VerificationError, content_hash, ed25519::Ed25519Verifier,
    sign::FIELDS_TO_REMOVE_FOR_SIGNING,
};

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
/// * `fetch_public_keys`: A type to get the public signing keys of servers by key ID.
///   [`required_server_signatures_to_verify_event()`] can be called to get the list of servers that
///   must appear in this map.
/// * `object`: The JSON object of the event that was signed.
/// * `room_version`: The version of the event's room.
///
/// # Examples
///
/// ```rust
/// use ruma_common::{RoomVersionId, serde::Base64};
/// use ruma_signatures::{PublicKeyMap, PublicKeySet, Verified, verify_event};
///
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
///     }"#,
/// )?;
///
/// // Create the `PublicKeyMap` that will inform `verify_json` which signatures to verify.
/// let mut public_key_set = PublicKeySet::new();
/// public_key_set.insert("ed25519:1".into(), Base64::parse(PUBLIC_KEY.to_owned())?);
/// let mut public_key_map = PublicKeyMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Get the redaction rules for the version of the current room.
/// let rules =
///     RoomVersionId::V6.rules().expect("The rules should be known for a supported room version");
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// let verification_result = verify_event(&public_key_map, &object, &rules)?;
/// assert_eq!(verification_result, Verified::All);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_event(
    fetch_public_keys: &impl FetchEntityPublicSigningKey,
    object: &CanonicalJsonObject,
    rules: &RoomVersionRules,
) -> Result<Verified, VerificationError> {
    let redacted = redact(object.clone(), &rules.redaction, None)?;

    let hashes = object.get_as_required_object("hashes", "hashes")?;
    let hash = hashes.get_as_required_string("sha256", "hashes.sha256")?;
    let signature_map = object.get_as_required_object("signatures", "signatures")?;

    let servers_to_check = required_server_signatures_to_verify_event(object, &rules.signatures)?;
    let canonical_json = to_canonical_json_string_for_signing(&redacted)?;

    for entity_id in servers_to_check {
        verify_canonical_json_for_entity(
            entity_id.as_str(),
            fetch_public_keys,
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
/// * `fetch_public_keys`: A type to get the public signing keys of entities by key ID.
/// * `object`: The JSON object that was signed.
///
/// # Errors
///
/// Returns an error if verification fails.
///
/// # Examples
///
/// ```
/// use ruma_common::serde::Base64;
/// use ruma_signatures::{PublicKeyMap, PublicKeySet, verify_json};
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
///     }"#,
/// )?;
///
/// // Create the `PublicKeyMap` that will inform `verify_json` which signatures to verify.
/// let mut public_key_set = PublicKeySet::new();
/// public_key_set.insert("ed25519:1".into(), Base64::parse(PUBLIC_KEY.to_owned())?);
/// let mut public_key_map = PublicKeyMap::new();
/// public_key_map.insert("domain".into(), public_key_set);
///
/// // Verify at least one signature for each entity in `public_key_map`.
/// assert!(verify_json(&public_key_map, &object).is_ok());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_json(
    fetch_public_keys: &impl FetchEntityPublicSigningKey,
    object: &CanonicalJsonObject,
) -> Result<(), VerificationError> {
    let signature_map = object.get_as_required_object("signatures", "signatures")?;
    let canonical_json = to_canonical_json_string_for_signing(object)?;

    for entity_id in signature_map.keys() {
        verify_canonical_json_for_entity(
            entity_id,
            fetch_public_keys,
            signature_map,
            canonical_json.as_bytes(),
        )?;
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
///   [`to_canonical_json_string_for_signing()`].
///
/// # Errors
///
/// Returns an error if verification fails.
pub fn verify_canonical_json_bytes(
    algorithm: &SigningKeyAlgorithm,
    public_key: &[u8],
    signature: &[u8],
    canonical_json: &[u8],
) -> Result<(), VerificationError> {
    let verifier =
        verifier_from_algorithm(algorithm).ok_or(VerificationError::UnsupportedAlgorithm)?;

    verify_canonical_json_with(&verifier, public_key, signature, canonical_json)
}

/// Serialize the given JSON object to prepare it for [signing].
///
/// This serializes the object to [canonical JSON] form without the `signatures` and `unsigned`
/// fields.
///
/// # Parameters
///
/// * `object`: The JSON object to convert.
///
/// # Examples
///
/// ```
/// use ruma_signatures::to_canonical_json_string_for_signing;
///
/// let input = r#"{
///     "本": 2,
///     "日": 1
/// }"#;
///
/// let object = serde_json::from_str(input)?;
/// let canonical = to_canonical_json_string_for_signing(&object)?;
///
/// assert_eq!(canonical, r#"{"日":1,"本":2}"#);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [signing]: https://spec.matrix.org/v1.18/appendices/#signing-details
/// [canonical JSON]: https://spec.matrix.org/v1.18/appendices/#canonical-json
pub fn to_canonical_json_string_for_signing(
    object: &CanonicalJsonObject,
) -> Result<String, JsonError> {
    to_canonical_json_string_with_fields_to_remove(object, FIELDS_TO_REMOVE_FOR_SIGNING)
}

/// Serialize the given JSON object to the canonical JSON form without the given fields.
pub(crate) fn to_canonical_json_string_with_fields_to_remove(
    object: &CanonicalJsonObject,
    fields: &[&str],
) -> Result<String, JsonError> {
    let mut owned_object = object.clone();

    for field in fields {
        owned_object.remove(*field);
    }

    to_json_string(&owned_object).map_err(Into::into)
}

/// Uses a set of public keys to verify signed canonical JSON bytes for a given entity.
///
/// Implements the algorithm described in the spec for [checking signatures].
///
/// # Parameters
///
/// * `entity_id`: The entity to check the signatures for.
/// * `fetch_public_keys`: A type to get the public signing keys of servers by key ID.
/// * `signature_map`: The map of signatures from the signed JSON object.
/// * `canonical_json`: The signed canonical JSON bytes. Can be obtained by calling
///   [`to_canonical_json_string_for_signing()`].
///
/// # Errors
///
/// Returns an error if verification fails.
///
/// [checking signatures]: https://spec.matrix.org/v1.18/appendices/#checking-for-a-signature
fn verify_canonical_json_for_entity(
    entity_id: &str,
    fetch_public_keys: &impl FetchEntityPublicSigningKey,
    signature_map: &CanonicalJsonObject,
    canonical_json: &[u8],
) -> Result<(), VerificationError> {
    let signature_set = signature_map
        .get_as_object(entity_id, format!("signatures.{entity_id}"))?
        .ok_or_else(|| VerificationError::NoSignaturesForEntity(entity_id.to_owned()))?;

    let mut checked = false;
    for (key_id, signature) in signature_set {
        // If the key is not in the map of public keys, ignore.
        let Some(public_key) = fetch_public_keys.public_signing_key(entity_id, key_id) else {
            continue;
        };

        // If we cannot parse the key ID, ignore.
        let Ok(parsed_key_id) = <&SigningKeyId<AnyKeyName>>::try_from(key_id.as_str()) else {
            continue;
        };

        // If the signature uses an unknown algorithm, ignore.
        let Some(verifier) = verifier_from_algorithm(&parsed_key_id.algorithm()) else {
            continue;
        };

        let CanonicalJsonValue::String(signature) = signature else {
            return Err(CanonicalJsonFieldError::InvalidType {
                path: format!("signatures.{entity_id}.{key_id}"),
                expected: CanonicalJsonType::String,
                found: signature.json_type(),
            }
            .into());
        };

        let signature = Base64::<Standard>::parse(signature).map_err(|error| {
            VerificationError::InvalidBase64Signature {
                path: format!("signatures.{entity_id}.{key_id}"),
                source: error,
            }
        })?;

        verify_canonical_json_with(&verifier, public_key, signature.as_bytes(), canonical_json)?;
        checked = true;
    }

    if !checked {
        return Err(VerificationError::NoSupportedSignatureForEntity(entity_id.to_owned()));
    }

    Ok(())
}

/// Uses a public key to verify signed canonical JSON bytes.
///
/// # Parameters
///
/// * `verifier`: A [`Verifier`] appropriate for the digital signature algorithm that was used.
/// * `public_key`: The raw bytes of the public key used to sign the JSON.
/// * `signature`: The raw bytes of the signature.
/// * `canonical_json`: The signed canonical JSON bytes. Can be obtained by calling
///   [`to_canonical_json_string_for_signing()`].
///
/// # Errors
///
/// Returns an error if verification fails.
fn verify_canonical_json_with<V>(
    verifier: &V,
    public_key: &[u8],
    signature: &[u8],
    canonical_json: &[u8],
) -> Result<(), VerificationError>
where
    V: Verifier,
{
    verifier.verify_json(public_key, signature, canonical_json).map_err(Into::into)
}

/// Get the list of servers whose signature must be checked to verify the given event.
///
/// Applies the rules for [validating signatures on received events] for populating the list:
///
/// - Add the server of the `sender`, except if it's an invite event that results from a third-party
///   invite.
/// - For room versions 1 and 2, add the server of the `event_id`.
/// - For room versions that support restricted join rules, if it's a join event with a
///   `join_authorised_via_users_server`, add the server of that user.
///
/// [validating signatures on received events]: https://spec.matrix.org/v1.18/server-server-api/#validating-hashes-and-signatures-on-received-events
pub fn required_server_signatures_to_verify_event(
    object: &CanonicalJsonObject,
    rules: &SignaturesRules,
) -> Result<BTreeSet<OwnedServerName>, VerificationError> {
    let mut servers_to_check = BTreeSet::new();

    if !is_invite_via_third_party_id(object)? {
        let sender = object.get_as_required_string("sender", "sender")?;
        let user_id = <&UserId>::try_from(sender).map_err(|source| {
            VerificationError::ParseIdentifier { identifier_type: "user ID", source }
        })?;

        servers_to_check.insert(user_id.server_name().to_owned());
    }

    if rules.check_event_id_server {
        let raw_event_id = object.get_as_required_string("event_id", "event_id")?;
        let event_id: OwnedEventId = raw_event_id.parse().map_err(|source| {
            VerificationError::ParseIdentifier { identifier_type: "event ID", source }
        })?;

        let server_name = event_id.server_name().map(ToOwned::to_owned).ok_or_else(|| {
            VerificationError::ParseIdentifier {
                identifier_type: "event ID",
                source: IdParseError::InvalidServerName,
            }
        })?;

        servers_to_check.insert(server_name);
    }

    if rules.check_join_authorised_via_users_server
        && let Some(authorized_user) = object
            .get("content")
            .and_then(|c| c.as_object())
            .map(|c| {
                c.get_as_string(
                    "join_authorised_via_users_server",
                    "content.join_authorised_via_users_server",
                )
            })
            .transpose()?
            .flatten()
    {
        let authorized_user = <&UserId>::try_from(authorized_user).map_err(|source| {
            VerificationError::ParseIdentifier { identifier_type: "user ID", source }
        })?;

        servers_to_check.insert(authorized_user.server_name().to_owned());
    }

    Ok(servers_to_check)
}

/// Whether the given event is an `m.room.member` invite that was created as the result of a
/// third-party invite.
///
/// Returns an error if the object has not the expected format of an `m.room.member` event.
fn is_invite_via_third_party_id(object: &CanonicalJsonObject) -> Result<bool, JsonError> {
    let event_type = object.get_as_required_string("type", "type")?;

    if event_type != "m.room.member" {
        return Ok(false);
    }

    let content = object.get_as_required_object("content", "content")?;
    let membership = content.get_as_required_string("membership", "content.membership")?;

    if membership != "invite" {
        return Ok(false);
    }

    Ok(content.get_as_object("third_party_invite", "content.third_party_invite")?.is_some())
}

/// A digital signature verifier.
pub(crate) trait Verifier {
    /// The error type returned by the verifier.
    type Error: std::error::Error + Into<VerificationError>;

    /// Use a public key to verify a signature against the JSON object that was signed.
    ///
    /// # Parameters
    ///
    /// * `public_key`: The raw bytes of the public key of the key pair used to sign the message.
    /// * `signature`: The raw bytes of the signature to verify.
    /// * `message`: The raw bytes of the message that was signed.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    fn verify_json(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), Self::Error>;
}

/// Get the verifier for the given algorithm, if it is supported.
fn verifier_from_algorithm(algorithm: &SigningKeyAlgorithm) -> Option<impl Verifier + use<>> {
    match algorithm {
        SigningKeyAlgorithm::Ed25519 => Some(Ed25519Verifier),
        _ => None,
    }
}

/// A value returned when an event is successfully verified.
///
/// Event verification involves verifying both signatures and a content hash. It is possible for
/// the signatures on an event to be valid, but for the hash to be different than the one
/// calculated during verification. This is not necessarily an error condition, as it may indicate
/// that the event has been redacted. In this case, receiving homeservers should store a redacted
/// version of the event.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum Verified {
    /// All signatures are valid and the content hashes match.
    All,

    /// All signatures are valid but the content hashes don't match.
    ///
    /// This may indicate a redacted event.
    Signatures,
}

/// A map from entity names to sets of public keys for that entity.
///
/// An entity is generally a homeserver, e.g. `example.com`.
pub type PublicKeyMap = BTreeMap<String, PublicKeySet>;

/// A set of public keys for a single homeserver.
///
/// This is represented as a map from key ID to base64-encoded signature.
pub type PublicKeySet = BTreeMap<String, Base64>;

/// A trait implemented by types that allow to get the public signing keys for a given entity.
pub trait FetchEntityPublicSigningKey {
    /// Get the bytes of the public signing key with the given ID for the given entity.
    fn public_signing_key(&self, entity: &str, key_id: &str) -> Option<&[u8]>;
}

impl<T> FetchEntityPublicSigningKey for &T
where
    T: FetchEntityPublicSigningKey,
{
    fn public_signing_key(&self, entity: &str, key_id: &str) -> Option<&[u8]> {
        (*self).public_signing_key(entity, key_id)
    }
}

impl<Entity, KeyId> FetchEntityPublicSigningKey for BTreeMap<Entity, BTreeMap<KeyId, Base64>>
where
    Entity: std::borrow::Borrow<str> + Ord,
    KeyId: std::borrow::Borrow<str> + Ord,
{
    fn public_signing_key(&self, entity: &str, key_id: &str) -> Option<&[u8]> {
        self.get(entity)?.get(key_id).map(Base64::as_bytes)
    }
}
