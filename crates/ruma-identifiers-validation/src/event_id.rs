use crate::{validate_delimited_id, Error};

pub fn validate(s: &str) -> Result<(), Error> {
    if s.contains(':') {
        validate_delimited_id(s, &['$'])?;
    } else if !s.starts_with('$') {
        return Err(Error::MissingLeadingSigil);
    }

    Ok(())
}
