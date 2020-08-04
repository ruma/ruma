use std::num::NonZeroU8;

use crate::{parse_id, Error};

pub fn validate(s: &str) -> Result<NonZeroU8, Error> {
    parse_id(s, &['#', '!'])
}
