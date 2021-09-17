use crate::Error;

pub fn validate(s: &str) -> Result<(), Error> {
    let colon_idx = s.find(':').ok_or(Error::MissingDelimiter)?;

    if colon_idx == 0 {
        Err(Error::InvalidKeyAlgorithm)
    } else {
        // Any non-empty string is accepted as a key algorithm for forwards compatibility
        Ok(())
    }
}
