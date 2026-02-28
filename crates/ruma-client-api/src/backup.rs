//! Endpoints for server-side key backups.

pub mod add_backup_keys;
pub mod add_backup_keys_for_room;
pub mod add_backup_keys_for_session;
pub mod create_backup_version;
pub mod delete_backup_keys;
pub mod delete_backup_keys_for_room;
pub mod delete_backup_keys_for_session;
pub mod delete_backup_version;
pub mod get_backup_info;
pub mod get_backup_keys;
pub mod get_backup_keys_for_room;
pub mod get_backup_keys_for_session;
pub mod get_latest_backup_info;
pub mod update_backup_version;

use std::{borrow::Cow, collections::BTreeMap};

use js_int::UInt;
use ruma_common::{
    CrossSigningOrDeviceSignatures,
    serde::{Base64, JsonObject, Raw},
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// A wrapper around a mapping of session IDs to key data.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomKeyBackup {
    /// A map of session IDs to key data.
    pub sessions: BTreeMap<String, Raw<KeyBackupData>>,
}

impl RoomKeyBackup {
    /// Creates a new `RoomKeyBackup` with the given sessions.
    pub fn new(sessions: BTreeMap<String, Raw<KeyBackupData>>) -> Self {
        Self { sessions }
    }
}

/// The algorithm used for storing backups.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "algorithm", content = "auth_data")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum BackupAlgorithm {
    /// `m.megolm_backup.v1.curve25519-aes-sha2` backup algorithm.
    #[serde(rename = "m.megolm_backup.v1.curve25519-aes-sha2")]
    MegolmBackupV1Curve25519AesSha2(MegolmBackupV1Curve25519AesSha2AuthData),

    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomBackupAlgorithm),
}

impl BackupAlgorithm {
    /// Returns a reference to the `algorithm` string.
    pub fn algorithm(&self) -> &str {
        match self {
            Self::MegolmBackupV1Curve25519AesSha2(_) => "m.megolm_backup.v1.curve25519-aes-sha2",
            Self::_Custom(c) => &c.algorithm,
        }
    }

    /// Returns the data of the algorithm.
    ///
    /// Prefer to use the public variants of `BackupAlgorithm` where possible; this method is meant
    /// to be used for custom algorithms only.
    pub fn auth_data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(obj: &T) -> JsonObject {
            match serde_json::to_value(obj).expect("backup data serialization to succeed") {
                JsonValue::Object(obj) => obj,
                _ => panic!("all backup data types must serialize to objects"),
            }
        }

        match self {
            Self::MegolmBackupV1Curve25519AesSha2(d) => Cow::Owned(serialize(d)),
            Self::_Custom(c) => Cow::Borrowed(&c.auth_data),
        }
    }
}

impl From<MegolmBackupV1Curve25519AesSha2AuthData> for BackupAlgorithm {
    fn from(value: MegolmBackupV1Curve25519AesSha2AuthData) -> Self {
        Self::MegolmBackupV1Curve25519AesSha2(value)
    }
}

/// The data for the `m.megolm_backup.v1.curve25519-aes-sha2` backup algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct MegolmBackupV1Curve25519AesSha2AuthData {
    /// The curve25519 public key used to encrypt the backups, encoded in unpadded base64.
    pub public_key: Base64,

    /// Signatures of the auth_data as Signed JSON.
    pub signatures: CrossSigningOrDeviceSignatures,
}

impl MegolmBackupV1Curve25519AesSha2AuthData {
    /// Construct a new `MegolmBackupV1Curve25519AesSha2BackupAlgorithm` using the given public key.
    pub fn new(public_key: Base64) -> Self {
        Self { public_key, signatures: Default::default() }
    }
}

/// The payload for a custom backup algorithm.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomBackupAlgorithm {
    /// The backup algorithm.
    algorithm: String,

    /// The data of the algorithm.
    auth_data: JsonObject,
}

/// Information about the backup key.
///
/// To create an instance of this type, first create a [`KeyBackupDataInit`] and convert it via
/// `KeyBackupData::from` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct KeyBackupData {
    /// The index of the first message in the session that the key can decrypt.
    pub first_message_index: UInt,

    /// The number of times this key has been forwarded via key-sharing between devices.
    pub forwarded_count: UInt,

    /// Whether the device backing up the key verified the device that the key is from.
    pub is_verified: bool,

    /// Encrypted data about the session.
    pub session_data: EncryptedSessionData,
}

/// Information about the backup key.
///
/// This struct will not be updated even if additional fields are added to [`KeyBackupData`] in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct KeyBackupDataInit {
    /// The index of the first message in the session that the key can decrypt.
    pub first_message_index: UInt,

    /// The number of times this key has been forwarded via key-sharing between devices.
    pub forwarded_count: UInt,

    /// Whether the device backing up the key verified the device that the key is from.
    pub is_verified: bool,

    /// Encrypted data about the session.
    pub session_data: EncryptedSessionData,
}

impl From<KeyBackupDataInit> for KeyBackupData {
    fn from(init: KeyBackupDataInit) -> Self {
        let KeyBackupDataInit { first_message_index, forwarded_count, is_verified, session_data } =
            init;
        Self { first_message_index, forwarded_count, is_verified, session_data }
    }
}

/// The encrypted algorithm-dependent data for backups.
///
/// To create an instance of this type, first create an [`EncryptedSessionDataInit`] and convert it
/// via `EncryptedSessionData::from` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct EncryptedSessionData {
    /// Unpadded base64-encoded public half of the ephemeral key.
    pub ephemeral: Base64,

    /// Ciphertext, encrypted using AES-CBC-256 with PKCS#7 padding, encoded in base64.
    pub ciphertext: Base64,

    /// First 8 bytes of MAC key, encoded in base64.
    pub mac: Base64,
}

/// The encrypted algorithm-dependent data for backups.
///
/// This struct will not be updated even if additional fields are added to [`EncryptedSessionData`]
/// in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct EncryptedSessionDataInit {
    /// Unpadded base64-encoded public half of the ephemeral key.
    pub ephemeral: Base64,

    /// Ciphertext, encrypted using AES-CBC-256 with PKCS#7 padding, encoded in base64.
    pub ciphertext: Base64,

    /// First 8 bytes of MAC key, encoded in base64.
    pub mac: Base64,
}

impl From<EncryptedSessionDataInit> for EncryptedSessionData {
    fn from(init: EncryptedSessionDataInit) -> Self {
        let EncryptedSessionDataInit { ephemeral, ciphertext, mac } = init;
        Self { ephemeral, ciphertext, mac }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use assert_matches2::assert_matches;
    use ruma_common::{
        SigningKeyAlgorithm, SigningKeyId, canonical_json::assert_to_canonical_json_eq,
        owned_user_id, serde::Base64,
    };
    use serde_json::{Value as JsonValue, from_value as from_json_value, json};

    use super::{BackupAlgorithm, MegolmBackupV1Curve25519AesSha2AuthData};

    #[test]
    fn megolm_v1_backup_algorithm_serialize_roundtrip() {
        let json = json!({
            "algorithm": "m.megolm_backup.v1.curve25519-aes-sha2",
            "auth_data": {
                "public_key": "YWJjZGVm",
                "signatures": {
                    "@alice:example.org": {
                        "ed25519:DEVICEID": "signature",
                    },
                },
            },
        });

        let mut backup_algorithm =
            MegolmBackupV1Curve25519AesSha2AuthData::new(Base64::new(b"abcdef".to_vec()));
        backup_algorithm.signatures.insert_signature(
            owned_user_id!("@alice:example.org"),
            SigningKeyId::from_parts(SigningKeyAlgorithm::Ed25519, "DEVICEID".into()),
            "signature".to_owned(),
        );
        assert_to_canonical_json_eq!(BackupAlgorithm::from(backup_algorithm), json.clone());

        assert_matches!(
            from_json_value(json),
            Ok(BackupAlgorithm::MegolmBackupV1Curve25519AesSha2(auth_data))
        );
        assert_eq!(auth_data.public_key.as_bytes(), b"abcdef");
        assert_matches!(
            auth_data.signatures.get(&owned_user_id!("@alice:example.org")),
            Some(user_signatures)
        );

        let mut user_signatures_iter = user_signatures.iter();
        assert_matches!(user_signatures_iter.next(), Some((key_id, signature)));
        assert_eq!(key_id, "ed25519:DEVICEID");
        assert_eq!(signature, "signature");
        assert_matches!(user_signatures_iter.next(), None);
    }

    #[test]
    fn custom_backup_algorithm_serialize_roundtrip() {
        let json = json!({
            "algorithm": "local.dev.unknown_algorithm",
            "auth_data": {
                "foo": "bar",
                "signatures": {
                    "ed25519:DEVICEID": "signature",
                },
            },
        });

        assert_matches!(from_json_value::<BackupAlgorithm>(json.clone()), Ok(backup_algorithm));
        assert_eq!(backup_algorithm.algorithm(), "local.dev.unknown_algorithm");
        assert_matches!(backup_algorithm.auth_data(), Cow::Borrowed(auth_data));

        assert_matches!(auth_data.get("foo"), Some(JsonValue::String(foo)));
        assert_eq!(foo, "bar");
        assert_matches!(auth_data.get("signatures"), Some(JsonValue::Object(signatures)));
        assert_matches!(signatures.get("ed25519:DEVICEID"), Some(JsonValue::String(signature)));
        assert_eq!(signature, "signature");

        assert_to_canonical_json_eq!(backup_algorithm, json);
    }
}
