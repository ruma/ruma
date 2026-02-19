use crate::Error;

pub const fn validate(s: &str) -> Result<(), Error> {
    if s.len() > 255 {
        return Err(Error::MaximumLengthExceeded);
    } else if contains_invalid_byte(s.as_bytes()) {
        return Err(Error::InvalidCharacters);
    } else if s.is_empty() {
        return Err(Error::Empty);
    }

    Ok(())
}

const fn contains_invalid_byte(mut bytes: &[u8]) -> bool {
    // non-const form:
    //
    // bytes.iter().all(|b| b.is_ascii_alphanumeric() || b".=_-".contains(&b))
    loop {
        if let Some((byte, rest)) = bytes.split_first() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'=' | b'_' | b'-') {
                bytes = rest;
            } else {
                break true;
            }
        } else {
            break false;
        }
    }
}
