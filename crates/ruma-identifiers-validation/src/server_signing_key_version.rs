use crate::Error;

#[cfg_attr(feature = "compat-server-signing-key-version", allow(unused_variables))]
pub fn validate(s: &str) -> Result<(), Error> {
    #[cfg(not(feature = "compat-server-signing-key-version"))]
    {
        if s.is_empty() {
            return Err(Error::Empty);
        } else if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(Error::InvalidCharacters);
        }
    }

    Ok(())
}
