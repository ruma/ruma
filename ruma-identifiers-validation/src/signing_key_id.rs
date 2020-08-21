use std::{num::NonZeroU8, str::FromStr};

use crate::{crypto_algorithms::SigningKeyAlgorithm, Error};

pub fn validate(s: &str) -> Result<NonZeroU8, Error> {
    let colon_idx = NonZeroU8::new(s.find(':').ok_or(Error::MissingSigningKeyDelimiter)? as u8)
        .ok_or(Error::UnknownKeyAlgorithm)?;

    validate_signing_key_algorithm(&s[..colon_idx.get() as usize])?;
    validate_version(&s[colon_idx.get() as usize + 1..])?;

    Ok(colon_idx)
}

fn validate_version(version: &str) -> Result<(), Error> {
    if version.is_empty() {
        return Err(Error::EmptyRoomVersionId);
    } else if !version.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}

fn validate_signing_key_algorithm(algorithm: &str) -> Result<(), Error> {
    match SigningKeyAlgorithm::from_str(algorithm) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::UnknownKeyAlgorithm),
    }
}
