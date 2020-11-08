use std::num::NonZeroU8;

use crate::{parse_id, Error};

pub fn validate(s: &str) -> Result<Option<NonZeroU8>, Error> {
    Ok(match s.contains(':') {
        true => Some(parse_id(s, &['$'])?),
        false => {
            if !s.starts_with('$') {
                return Err(Error::MissingLeadingSigil);
            }

            None
        }
    })
}
