use js_int::{uint, UInt};

use crate::{error::VoipVersionIdError, Error};

pub fn validate(u: UInt) -> Result<(), Error> {
    if u != uint!(0) {
        return Err(VoipVersionIdError::WrongUintValue.into());
    }

    Ok(())
}
