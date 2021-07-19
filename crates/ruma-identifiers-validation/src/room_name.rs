use crate::Error;

pub fn validate(value: &str) -> Result<(), Error> {
    match value.len() {
        0 => Err(Error::EmptyRoomName),
        1..=255 => Ok(()),
        _ => Err(Error::MaximumLengthExceeded),
    }
}
