use js_int::{UInt, uint};

use crate::{Error, error::VoipVersionIdError};

pub fn validate(u: UInt) -> Result<(), Error> {
    if u != uint!(0) {
        return Err(VoipVersionIdError::WrongUintValue.into());
    }

    Ok(())
}
