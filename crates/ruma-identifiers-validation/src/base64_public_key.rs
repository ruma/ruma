use crate::Error;

pub fn validate(s: &str) -> Result<(), Error> {
    if s.is_empty() {
        return Err(Error::Empty);
    } else if !s.chars().all(|c| c.is_alphanumeric() || matches!(c, '+' | '/' | '=')) {
        return Err(Error::InvalidCharacters);
    }
    Ok(())
}
