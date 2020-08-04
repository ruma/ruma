use std::num::NonZeroU8;

use crate::{parse_id, validate_id, Error};

pub fn validate(s: &str) -> Result<Option<NonZeroU8>, Error> {
    Ok(match s.contains(':') {
        true => Some(parse_id(s, &['$'])?),
        false => {
            validate_id(s, &['$'])?;
            None
        }
    })
}
