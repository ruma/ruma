use std::{borrow::Cow, collections::BTreeMap, mem};

use ruma_common::{
    AnyKeyName, CanonicalJsonObject, CanonicalJsonValue, OwnedSigningKeyId, SigningKeyAlgorithm,
    canonical_json::{CanonicalJsonType, redact},
    room_version_rules::RedactionRules,
    serde::{Base64, base64::Standard},
};
use serde_json::to_string as to_json_string;

use crate::{JsonError, content_hash};

/// The fields to remove from a JSON object when serializing it for signing.
pub(crate) static FIELDS_TO_REMOVE_FOR_SIGNING: &[&str] = &["signatures", "unsigned"];

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
/// # use ruma_signatures::{hash_and_sign_event, ed25519::Ed25519KeyPair};
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
) -> Result<(), JsonError>
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
        _ => {
            return Err(JsonError::InvalidType {
                path: "hashes".to_owned(),
                expected: CanonicalJsonType::Object,
                found: hashes_value.json_type(),
            });
        }
    };

    let mut redacted = redact(object.clone(), redaction_rules, None).map_err(JsonError::from)?;

    sign_json(entity_id, key_pair, &mut redacted)?;

    object.insert("signatures".into(), mem::take(redacted.get_mut("signatures").unwrap()));

    Ok(())
}

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
/// let key_pair = ruma_signatures::ed25519::Ed25519KeyPair::from_der(
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
) -> Result<(), JsonError>
where
    K: KeyPair,
{
    let (signatures_key, mut signature_map) = match object.remove_entry("signatures") {
        Some((key, CanonicalJsonValue::Object(signatures))) => (Cow::Owned(key), signatures),
        Some((_, value)) => {
            return Err(JsonError::InvalidType {
                path: "signatures".to_owned(),
                expected: CanonicalJsonType::Object,
                found: value.json_type(),
            });
        }
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
        return Err(JsonError::InvalidType {
            path: format!("signatures.{entity_id}"),
            expected: CanonicalJsonType::Object,
            found: signature_set.json_type(),
        });
    };

    signature_set.insert(signature.id(), CanonicalJsonValue::String(signature.base64()));

    // Put `signatures` and `unsigned` back in.
    object.insert(signatures_key.into(), CanonicalJsonValue::Object(signature_map));

    if let Some((k, v)) = maybe_unsigned_entry {
        object.insert(k, v);
    }

    Ok(())
}

/// A cryptographic key pair for digitally signing data.
pub trait KeyPair: Sized {
    /// Signs a JSON object.
    ///
    /// # Parameters
    ///
    /// * `message`: An arbitrary series of bytes to sign.
    fn sign(&self, message: &[u8]) -> Signature;
}

/// A digital signature.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Signature {
    /// The ID of the key used to generate this signature.
    pub(crate) key_id: OwnedSigningKeyId<AnyKeyName>,

    /// The signature data.
    pub(crate) signature: Vec<u8>,
}

impl Signature {
    /// Creates a signature from raw bytes.
    ///
    /// This constructor will ensure that the key ID has the correct `algorithm:key_name` format.
    ///
    /// # Parameters
    ///
    /// * `key_id`: A key identifier, e.g. `ed25519:1`.
    /// * `signature`: The digital signature, as a series of bytes.
    pub fn new(key_id: OwnedSigningKeyId<AnyKeyName>, signature: Vec<u8>) -> Self {
        Self { key_id, signature }
    }

    /// The algorithm used to generate the signature.
    pub fn algorithm(&self) -> SigningKeyAlgorithm {
        self.key_id.algorithm()
    }

    /// The raw bytes of the signature.
    pub fn as_bytes(&self) -> &[u8] {
        self.signature.as_slice()
    }

    /// A base64 encoding of the signature.
    ///
    /// Uses the standard character set with no padding.
    pub fn base64(&self) -> String {
        Base64::<Standard, _>::new(self.signature.as_slice()).encode()
    }

    /// The key identifier, a string containing the signature algorithm and the key "version"
    /// separated by a colon, e.g. `ed25519:1`.
    pub fn id(&self) -> String {
        self.key_id.to_string()
    }

    /// The "version" of the key used for this signature.
    ///
    /// Versions are used as an identifier to distinguish signatures generated from different keys
    /// but using the same algorithm on the same homeserver.
    pub fn version(&self) -> &str {
        self.key_id.key_name().as_ref()
    }

    /// Split this `Signature` into its key identifier and bytes.
    pub fn into_parts(self) -> (OwnedSigningKeyId<AnyKeyName>, Vec<u8>) {
        (self.key_id, self.signature)
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::SigningKeyAlgorithm;

    use super::Signature;

    #[test]
    fn valid_key_id() {
        let signature = Signature::new("ed25519:abcdef".try_into().unwrap(), vec![]);
        assert_eq!(signature.algorithm(), SigningKeyAlgorithm::Ed25519);
        assert_eq!(signature.version(), "abcdef");
    }

    #[test]
    fn unknown_key_id_algorithm() {
        let signature = Signature::new("foobar:abcdef".try_into().unwrap(), vec![]);
        assert_eq!(signature.algorithm().as_str(), "foobar");
        assert_eq!(signature.version(), "abcdef");
    }
}
