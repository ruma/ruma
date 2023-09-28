use crate::Error;

pub fn validate(s: &str) -> Result<(), Error> {
    match s.as_bytes().first() {
        Some(b'#') => crate::room_alias_id::validate(s),
        Some(b'!') => crate::room_id::validate(s),
        _ => Err(Error::MissingLeadingSigil),
    }
}
