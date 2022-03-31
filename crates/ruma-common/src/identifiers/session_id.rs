//! Matrix session ID.

/// A session ID.
///
/// Session IDs in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]`. Their length must
/// must not exceed 255 characters.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(str);

owned_identifier!(OwnedSessionId, SessionId);

opaque_identifier_validated!(
    SessionId,
    OwnedSessionId,
    ruma_identifiers_validation::session_id::validate
);
