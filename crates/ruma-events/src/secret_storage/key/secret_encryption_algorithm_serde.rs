use ruma_common::serde::JsonObject;
use serde::{Deserialize, Serialize, de};

use super::{
    CustomSecretEncryptionAlgorithm, SecretStorageEncryptionAlgorithm,
    SecretStorageV1AesHmacSha2Properties,
};

#[derive(Deserialize)]
#[serde(untagged)]
enum SecretStorageEncryptionAlgorithmDeHelper {
    Known(KnownSecretStorageEncryptionAlgorithmDeHelper),
    Unknown(UnknownSecretStorageEncryptionAlgorithmDeHelper),
}

#[derive(Deserialize)]
#[serde(tag = "algorithm")]
enum KnownSecretStorageEncryptionAlgorithmDeHelper {
    #[serde(rename = "m.secret_storage.v1.aes-hmac-sha2")]
    V1AesHmacSha2(SecretStorageV1AesHmacSha2Properties),
}

#[derive(Deserialize)]
struct UnknownSecretStorageEncryptionAlgorithmDeHelper {
    /// The encryption algorithm to be used for the key.
    algorithm: String,

    /// Algorithm-specific properties.
    #[serde(flatten)]
    properties: JsonObject,
}

impl<'de> Deserialize<'de> for SecretStorageEncryptionAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let helper = SecretStorageEncryptionAlgorithmDeHelper::deserialize(deserializer)?;

        Ok(match helper {
            SecretStorageEncryptionAlgorithmDeHelper::Known(k) => match k {
                KnownSecretStorageEncryptionAlgorithmDeHelper::V1AesHmacSha2(p) => {
                    Self::V1AesHmacSha2(p)
                }
            },
            SecretStorageEncryptionAlgorithmDeHelper::Unknown(
                UnknownSecretStorageEncryptionAlgorithmDeHelper { algorithm, properties },
            ) => Self::_Custom(CustomSecretEncryptionAlgorithm { algorithm, properties }),
        })
    }
}

#[derive(Debug, Serialize)]
struct SecretStorageEncryptionAlgorithmSerHelper<'a, T: Serialize> {
    algorithm: &'a str,
    #[serde(flatten)]
    properties: T,
}

impl Serialize for SecretStorageEncryptionAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::V1AesHmacSha2(properties) => {
                let algorithm = self.algorithm();
                SecretStorageEncryptionAlgorithmSerHelper { algorithm, properties }
                    .serialize(serializer)
            }
            Self::_Custom(CustomSecretEncryptionAlgorithm { algorithm, properties }) => {
                SecretStorageEncryptionAlgorithmSerHelper { algorithm, properties }
                    .serialize(serializer)
            }
        }
    }
}
