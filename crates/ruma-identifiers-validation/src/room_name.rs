use crate::Error;

pub fn validate(value: &str) -> Result<(), Error> {
    match value.len() {
        0 => Err(Error::Empty),
        1..=255 => Ok(()),
        _ => Err(Error::MaximumLengthExceeded),
    }
}
