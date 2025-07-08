use crate::{Error, validate_delimited_id};

pub fn validate(s: &str) -> Result<(), Error> {
    if s.contains(':') {
        validate_delimited_id(s, b'$')?;
    } else if !s.starts_with('$') {
        return Err(Error::MissingLeadingSigil);
    }

    Ok(())
}
