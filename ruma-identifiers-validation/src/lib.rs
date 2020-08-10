pub mod crypto_algorithms;
pub mod device_key_id;
pub mod error;
pub mod event_id;
pub mod room_alias_id;
pub mod room_id;
pub mod room_id_or_alias_id;
pub mod room_version_id;
pub mod server_key_id;
pub mod server_name;
pub mod user_id;

use std::num::NonZeroU8;

pub use error::Error;

/// All identifiers must be 255 bytes or less.
const MAX_BYTES: usize = 255;

/// The minimum number of characters an ID can be.
///
/// This is an optimization and not required by the spec. The shortest possible valid ID is a sigil
/// + a single character local ID + a colon + a single character hostname.
const MIN_CHARS: usize = 4;

/// Checks if an identifier is valid.
fn validate_id(id: &str, valid_sigils: &[char]) -> Result<(), Error> {
    if id.len() > MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    if id.len() < MIN_CHARS {
        return Err(Error::MinimumLengthNotSatisfied);
    }

    if !valid_sigils.contains(&id.chars().next().unwrap()) {
        return Err(Error::MissingSigil);
    }

    Ok(())
}

/// Checks an identifier that contains a localpart and hostname for validity,
/// and returns the index of the colon that separates the two.
fn parse_id(id: &str, valid_sigils: &[char]) -> Result<NonZeroU8, Error> {
    validate_id(id, valid_sigils)?;

    let colon_idx = id.find(':').ok_or(Error::MissingDelimiter)?;
    if colon_idx < 2 {
        return Err(Error::InvalidLocalPart);
    }

    server_name::validate(&id[colon_idx + 1..])?;

    Ok(NonZeroU8::new(colon_idx as u8).unwrap())
}
