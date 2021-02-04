//! Endpoints for server-side key backups.
#![cfg(feature = "unstable-pre-spec")]

pub mod add_backup_key_session;
pub mod add_backup_key_sessions;
pub mod add_backup_keys;
pub mod create_backup;
pub mod delete_backup;
pub mod delete_backup_key_session;
pub mod delete_backup_key_sessions;
pub mod delete_backup_keys;
pub mod get_backup;
pub mod get_backup_key_session;
pub mod get_backup_key_sessions;
pub mod get_backup_keys;
pub mod get_latest_backup;
pub mod update_backup;

use js_int::UInt;
use ruma_identifiers::{DeviceKeyId, UserId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A wrapper around a mapping of session IDs to key data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomKeyBackup {
    /// A map of session IDs to key data.
    pub sessions: BTreeMap<String, KeyBackupData>,
}

/// The algorithm used for storing backups.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "algorithm", content = "auth_data")]
pub enum BackupAlgorithm {
    /// `m.megolm_backup.v1.curve25519-aes-sha2` backup algorithm.
    #[serde(rename = "m.megolm_backup.v1.curve25519-aes-sha2")]
    MegolmBackupV1Curve25519AesSha2 {
        /// The curve25519 public key used to encrypt the backups, encoded in unpadded base64.
        public_key: String,

        /// Signatures of the auth_data as Signed JSON.
        signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,
    },
}

/// Information about the backup key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyBackupData {
    /// The index of the first message in the session that the key can decrypt.
    pub first_message_index: UInt,

    /// The number of times this key has been forwarded via key-sharing between devices.
    pub forwarded_count: UInt,

    /// Whether the device backing up the key verified the device that the key is from.
    pub is_verified: bool,

    /// Data about the session.
    pub session_data: SessionData,
}

/// The algorithm used for storing backups.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionData {
    /// Unpadded base64-encoded public half of the ephemeral key.
    pub ephemeral: String,

    /// Ciphertext, encrypted using AES-CBC-256 with PKCS#7 padding, encoded in base64.
    pub ciphertext: String,

    /// First 8 bytes of MAC key, encoded in base64.
    pub mac: String,
}
