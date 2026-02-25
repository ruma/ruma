use ruma_macros::ruma_id;

use super::{IdParseError, KeyName};

/// The name of a [one-time or fallback key].
///
/// One-time and fallback key names in Matrix are completely opaque character sequences. This
/// type is provided simply for its semantic value.
///
/// [one-time or fallback key]: https://spec.matrix.org/latest/client-server-api/#one-time-and-fallback-keys
#[ruma_id]
pub struct OneTimeKeyName;

impl KeyName for OneTimeKeyName {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}
