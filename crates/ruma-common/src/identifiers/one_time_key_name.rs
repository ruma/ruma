use ruma_macros::IdZst;

use super::{IdParseError, KeyName};

/// The name of a [one-time or fallback key].
///
/// One-time and fallback key names in Matrix are completely opaque character sequences. This
/// type is provided simply for its semantic value.
///
/// [one-time or fallback key]: https://spec.matrix.org/latest/client-server-api/#one-time-and-fallback-keys
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
pub struct OneTimeKeyName(str);

impl KeyName for OneTimeKeyName {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}

impl KeyName for OwnedOneTimeKeyName {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}
