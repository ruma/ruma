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
    } else if !version.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}
