use crate::{validate_id, Error};

pub fn validate(s: &str) -> Result<(), Error> {
    validate_id(s, b'!')
}
