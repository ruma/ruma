use crate::Error;

pub fn validate(s: &str) -> Result<(), Error> {
    if s.len() > 255 {
        return Err(Error::MaximumLengthExceeded);
    } else if !s.chars().all(|c| c.is_alphanumeric() || ".=_-".contains(c)) {
        return Err(Error::InvalidCharacters);
    } else if s.is_empty() {
        return Err(Error::Empty);
    }

    Ok(())
}
