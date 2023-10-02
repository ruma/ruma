use serde::{de, Deserialize, Serialize};

use super::{
    CustomSecretEncryptionAlgorithm, SecretStorageEncryptionAlgorithm,
    SecretStorageV1AesHmacSha2Properties,
};

#[derive(Deserialize)]
#[serde(untagged)]
enum SecretStorageEncryptionAlgorithmDeHelper {
    Known(KnownSecretStorageEncryptionAlgorithmDeHelper),
    Unknown(CustomSecretEncryptionAlgorithm),
}

#[derive(Deserialize)]
#[serde(tag = "algorithm")]
enum KnownSecretStorageEncryptionAlgorithmDeHelper {
    #[serde(rename = "m.secret_storage.v1.aes-hmac-sha2")]
    V1AesHmacSha2(SecretStorageV1AesHmacSha2Properties),
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
            SecretStorageEncryptionAlgorithmDeHelper::Unknown(c) => Self::_Custom(c),
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
        let algorithm = self.algorithm();
        match self {
            Self::V1AesHmacSha2(properties) => {
                SecretStorageEncryptionAlgorithmSerHelper { algorithm, properties }
                    .serialize(serializer)
            }
            Self::_Custom(properties) => {
                SecretStorageEncryptionAlgorithmSerHelper { algorithm, properties }
                    .serialize(serializer)
            }
        }
    }
}
