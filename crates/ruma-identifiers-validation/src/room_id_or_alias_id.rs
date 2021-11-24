use crate::{validate_delimited_id, Error};

pub fn validate(s: &str) -> Result<(), Error> {
    validate_delimited_id(s, &['#', '!'])
}
