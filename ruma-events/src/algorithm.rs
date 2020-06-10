use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

/// An encryption algorithm to be used to encrypt messages sent to a room.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(from = "String", into = "String")]
pub enum Algorithm {
    /// Olm version 1 using Curve25519, AES-256, and SHA-256.
    OlmV1Curve25519AesSha2,

    /// Megolm version 1 using AES-256 and SHA-256.
    MegolmV1AesSha2,

    /// Any algorithm that is not part of the specification.
    Custom(String),
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let algorithm_str = match *self {
            Algorithm::OlmV1Curve25519AesSha2 => "m.olm.v1.curve25519-aes-sha2",
            Algorithm::MegolmV1AesSha2 => "m.megolm.v1.aes-sha2",
            Algorithm::Custom(ref algorithm) => algorithm,
        };

        write!(f, "{}", algorithm_str)
    }
}

impl<T> From<T> for Algorithm
where
    T: Into<String> + AsRef<str>,
{
    fn from(s: T) -> Algorithm {
        match s.as_ref() {
            "m.olm.v1.curve25519-aes-sha2" => Algorithm::OlmV1Curve25519AesSha2,
            "m.megolm.v1.aes-sha2" => Algorithm::MegolmV1AesSha2,
            _ => Algorithm::Custom(s.into()),
        }
    }
}

impl From<Algorithm> for String {
    fn from(algorithm: Algorithm) -> String {
        algorithm.to_string()
    }
}

#[cfg(test)]
mod tests {
    use ruma_serde::test::serde_json_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_and_deserialize_from_display_form() {
        serde_json_eq(Algorithm::MegolmV1AesSha2, json!("m.megolm.v1.aes-sha2"));
        serde_json_eq(
            Algorithm::OlmV1Curve25519AesSha2,
            json!("m.olm.v1.curve25519-aes-sha2"),
        );
        serde_json_eq(
            Algorithm::Custom("io.ruma.test".to_string()),
            json!("io.ruma.test"),
        );
    }
}
