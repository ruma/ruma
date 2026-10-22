use crate::{Error, KeyName};

pub fn validate<K: KeyName + ?Sized>(s: &str) -> Result<(), Error> {
    let colon_idx = s.find(':').ok_or(Error::MissingColon)?;
    if colon_idx == 0 {
        return Err(Error::Empty);
    }
    K::validate(&s[colon_idx + 1..])
}
