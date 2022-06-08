use crate::Error;

pub fn validate(s: &str) -> Result<(), Error> {
    if s.len() > 255 {
        return Err(Error::MaximumLengthExceeded);
    } else if !s.bytes().all(|b| b.is_ascii_alphanumeric() || b".=_-".contains(&b)) {
        return Err(Error::InvalidCharacters);
    } else if s.is_empty() {
        return Err(Error::Empty);
    }

    Ok(())
}
