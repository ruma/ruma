//! Matrix session ID.

use ruma_identifiers_validation::session_id::validate;

opaque_identifier_validated! {
    /// A session ID.
    ///
    /// Session IDs in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]. Their length must
    /// must not exceed 255 characters.
    pub type SessionId;
    validate;
}
