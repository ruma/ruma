use crate::Error;

/// Room version identifiers cannot be more than 32 code points.
const MAX_CODE_POINTS: usize = 32;

pub fn validate(s: &str) -> Result<(), Error> {
    if s.is_empty() {
        Err(Error::EmptyRoomVersionId)
    } else if s.chars().count() > MAX_CODE_POINTS {
        Err(Error::MaximumLengthExceeded)
    } else {
        Ok(())
    }
}
