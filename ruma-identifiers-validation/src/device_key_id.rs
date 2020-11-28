use std::num::NonZeroU8;

use crate::Error;

pub fn validate(s: &str) -> Result<NonZeroU8, Error> {
    let colon_idx = NonZeroU8::new(s.find(':').ok_or(Error::MissingDelimiter)? as u8)
        .ok_or(Error::InvalidKeyAlgorithm)?;

    Ok(colon_idx)
}
