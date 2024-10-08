use ruma_macros::IdZst;

use super::{IdParseError, KeyName};

/// The version of a [homeserver signing key].
///
/// This is an opaque character sequences of `[a-zA-Z0-9_]`. This type is provided simply for its
/// semantic value.
///
/// With the `compat-server-signing-key-version` cargo feature, the validation of this type is
/// relaxed to accept any string.
///
/// [homeserver signing key]: https://spec.matrix.org/latest/server-server-api/#retrieving-server-keys
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(
    validate = ruma_identifiers_validation::server_signing_key_version::validate,
)]
pub struct ServerSigningKeyVersion(str);

impl KeyName for ServerSigningKeyVersion {
    fn validate(s: &str) -> Result<(), IdParseError> {
        ruma_identifiers_validation::server_signing_key_version::validate(s)
    }
}

impl KeyName for OwnedServerSigningKeyVersion {
    fn validate(s: &str) -> Result<(), IdParseError> {
        ruma_identifiers_validation::server_signing_key_version::validate(s)
    }
}
