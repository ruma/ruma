use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use serde::{Deserialize, Serialize};

/// An encryption algorithm to be used to encrypt messages sent to a room.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// Cow<str> because deserialization sometimes needs to copy to unescape things
#[serde(from = "Cow<'_, str>", into = "String")]
pub enum Algorithm {
    /// Olm version 1 using Curve25519, AES-256, and SHA-256.
    OlmV1Curve25519AesSha2,

    /// Megolm version 1 using AES-256 and SHA-256.
    MegolmV1AesSha2,

    /// Any algorithm that is not part of the specification.
    Custom(String),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to `ruma-events`.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let algorithm_str = match *self {
            Algorithm::OlmV1Curve25519AesSha2 => "m.olm.v1.curve25519-aes-sha2",
            Algorithm::MegolmV1AesSha2 => "m.megolm.v1.aes-sha2",
            Algorithm::Custom(ref algorithm) => algorithm,
            Algorithm::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.")
            }
        };

        write!(f, "{}", algorithm_str)
    }
}

impl From<Cow<'_, str>> for Algorithm {
    fn from(s: Cow<'_, str>) -> Algorithm {
        match &s as &str {
            "m.olm.v1.curve25519-aes-sha2" => Algorithm::OlmV1Curve25519AesSha2,
            "m.megolm.v1.aes-sha2" => Algorithm::MegolmV1AesSha2,
            _ => Algorithm::Custom(s.into_owned()),
        }
    }
}

impl From<&str> for Algorithm {
    fn from(s: &str) -> Algorithm {
        Algorithm::from(Cow::Borrowed(s))
    }
}

impl From<Algorithm> for String {
    fn from(algorithm: Algorithm) -> String {
        algorithm.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_and_deserialize_from_display_form() {
        serde_eq!(r#""m.megolm.v1.aes-sha2""#, Algorithm::MegolmV1AesSha2);
        serde_eq!(
            r#""m.olm.v1.curve25519-aes-sha2""#,
            Algorithm::OlmV1Curve25519AesSha2
        );
        serde_eq!(
            r#""io.ruma.test""#,
            Algorithm::Custom("io.ruma.test".to_string())
        );
    }
}
