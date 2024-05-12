use std::num::NonZeroU8;

use crate::Error;

pub fn validate(s: &str) -> Result<NonZeroU8, Error> {
    let colon_idx =
        NonZeroU8::new(s.find(':').ok_or(Error::MissingColon)? as u8).ok_or(Error::MissingColon)?;

    #[cfg(not(feature = "compat-key-id"))]
    validate_version(&s[colon_idx.get() as usize + 1..])?;

    Ok(colon_idx)
}

#[cfg(not(feature = "compat-key-id"))]
fn validate_version(version: &str) -> Result<(), Error> {
    if version.is_empty() {
        return Err(Error::Empty);
    }

    if !version.chars().all(is_valid_version_char) {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}

#[cfg(not(feature = "compat-key-id"))]
fn is_valid_version_char(c: char) -> bool {
    let is_valid = c.is_alphanumeric() || c == '_';

    #[cfg(feature = "compat-signature-id")]
    let is_valid = is_valid || c == '+' || c = '/';

    is_valid
}
