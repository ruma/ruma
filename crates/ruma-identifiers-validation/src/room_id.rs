use crate::{validate_id, Error};

/// Validate a [room ID] as used by clients.
///
/// [room ID]: https://spec.matrix.org/latest/appendices/#room-ids
pub fn validate(s: &str) -> Result<(), Error> {
    validate_id(s, b'!')?;

    // Since we cannot check the localpart, check at least the NUL byte.
    if s.as_bytes().contains(&b'\0') {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}
