//! Identifiers for homeserver signing keys used for federation.

use std::{convert::TryInto, num::NonZeroU8, str::FromStr};

use ruma_identifiers_validation::{key_algorithms::ServerKeyAlgorithm, Error};

/// Key identifiers used for homeserver signing keys.
#[derive(Clone, Debug)]
pub struct ServerKeyId {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
}

impl ServerKeyId {
    /// Create a `ServerKeyId` from a `ServerKeyAlgorithm` and a `ServerId`.
    pub fn from_parts(algorithm: ServerKeyAlgorithm, version: &str) -> Self {
        let algorithm: &str = algorithm.as_ref();

        let mut res = String::with_capacity(algorithm.len() + 1 + version.len());
        res.push_str(algorithm);
        res.push_str(":");
        res.push_str(version);

        let colon_idx =
            NonZeroU8::new(algorithm.len().try_into().expect("no algorithm name len > 255"))
                .expect("no empty algorithm name");

        ServerKeyId { full_id: res.into(), colon_idx }
    }

    /// Returns key algorithm of the server key ID.
    pub fn algorithm(&self) -> ServerKeyAlgorithm {
        ServerKeyAlgorithm::from_str(&self.full_id[..self.colon_idx.get() as usize]).unwrap()
    }

    /// Returns the version of the server key ID.
    pub fn version(&self) -> &str {
        &self.full_id[self.colon_idx.get() as usize + 1..]
    }
}

fn try_from<S>(key_id: S) -> Result<ServerKeyId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let colon_idx = ruma_identifiers_validation::server_key_id::validate(key_id.as_ref())?;
    Ok(ServerKeyId { full_id: key_id.into(), colon_idx })
}

common_impls!(ServerKeyId, try_from, "Key ID with algorithm and version");

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::{Error, ServerKeyId};

    #[cfg(feature = "serde")]
    use ruma_identifiers_validation::key_algorithms::ServerKeyAlgorithm;

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_id() {
        let server_key_id: ServerKeyId = from_json_value(json!("ed25519:Abc_1")).unwrap();
        assert_eq!(server_key_id.algorithm(), ServerKeyAlgorithm::Ed25519);
        assert_eq!(server_key_id.version(), "Abc_1");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_id() {
        let server_key_id: ServerKeyId = ServerKeyId::try_from("ed25519:abc123").unwrap();
        assert_eq!(to_json_value(&server_key_id).unwrap(), json!("ed25519:abc123"));
    }

    #[test]
    fn invalid_version_characters() {
        assert_eq!(ServerKeyId::try_from("ed25519:Abc-1").unwrap_err(), Error::InvalidCharacters);
    }

    #[test]
    fn invalid_key_algorithm() {
        assert_eq!(
            ServerKeyId::try_from("signed_curve25519:Abc-1").unwrap_err(),
            Error::UnknownKeyAlgorithm,
        );
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            ServerKeyId::try_from("ed25519|Abc_1").unwrap_err(),
            Error::MissingServerKeyDelimiter,
        );
    }
}
