use std::num::NonZeroU8;

use crate::Error;

pub fn validate(s: &str) -> Result<NonZeroU8, Error> {
    let colon_idx = NonZeroU8::new(s.find(':').ok_or(Error::MissingDelimiter)? as u8)
        .ok_or(Error::InvalidKeyAlgorithm)?;

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
