//! Identifiers for homeserver signing keys used for federation.

use std::{num::NonZeroU8, str::FromStr};

use crate::{error::Error, key_algorithms::ServerKeyAlgorithm};

/// Key identifiers used for homeserver signing keys.
#[derive(Clone, Debug)]
pub struct ServerKeyId {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
}

impl ServerKeyId {
    /// Returns key algorithm of the server key ID.
    pub fn algorithm(&self) -> ServerKeyAlgorithm {
        ServerKeyAlgorithm::from_str(&self.full_id.as_ref()[..self.colon_idx.get() as usize])
            .unwrap()
    }

    /// Returns the version of the server key ID.
    pub fn version(&self) -> &str {
        &self.full_id.as_ref()[self.colon_idx.get() as usize + 1..]
    }
}

fn try_from<S>(key_id: S) -> Result<ServerKeyId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let key_str = key_id.as_ref();
    let colon_idx =
        NonZeroU8::new(key_str.find(':').ok_or(Error::MissingServerKeyDelimiter)? as u8)
            .ok_or(Error::UnknownKeyAlgorithm)?;

    validate_server_key_algorithm(&key_str[..colon_idx.get() as usize])?;

    validate_version(&key_str[colon_idx.get() as usize + 1..])?;

    Ok(ServerKeyId { full_id: key_id.into(), colon_idx })
}

common_impls!(ServerKeyId, try_from, "Key ID with algorithm and version");

fn validate_version(version: &str) -> Result<(), Error> {
    if version.is_empty() {
        return Err(Error::MinimumLengthNotSatisfied);
    } else if !version.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}

fn validate_server_key_algorithm(algorithm: &str) -> Result<(), Error> {
    match ServerKeyAlgorithm::from_str(algorithm) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::UnknownKeyAlgorithm),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::{error::Error, ServerKeyId};

    #[cfg(feature = "serde")]
    use crate::key_algorithms::ServerKeyAlgorithm;

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
        assert_eq!(ServerKeyId::try_from("ed25519:Abc-1").unwrap_err(), Error::InvalidCharacters,);
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
