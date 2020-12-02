//! Error conditions.

use std::fmt::{self, Display, Formatter};

/// An error encountered when trying to parse an invalid ID string.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// The room version ID is empty.
    EmptyRoomVersionId,

    /// The ID's localpart contains invalid characters.
    ///
    /// Only relevant for user IDs.
    InvalidCharacters,

    /// The key algorithm is invalid (e.g. empty).
    InvalidKeyAlgorithm,

    /// The key version contains outside of [a-zA-Z0-9_].
    InvalidKeyVersion,

    /// The server name part of the the ID string is not a valid server name.
    InvalidServerName,

    /// The ID exceeds 255 bytes (or 32 codepoints for a room version ID).
    MaximumLengthExceeded,

    /// The ID is missing the colon delimiter between localpart and server name, or between key
    /// algorithm and key name / version.
    MissingDelimiter,

    /// The ID is missing the correct leading sigil.
    MissingLeadingSigil,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::EmptyRoomVersionId => "room version ID is empty",
            Error::InvalidCharacters => "localpart contains invalid characters",
            Error::InvalidKeyAlgorithm => "invalid key algorithm specified",
            Error::InvalidKeyVersion => "key ID version contains invalid characters",
            Error::InvalidServerName => "server name is not a valid IP address or domain name",
            Error::MaximumLengthExceeded => "ID exceeds 255 bytes",
            Error::MissingDelimiter => "required colon is missing",
            Error::MissingLeadingSigil => "leading sigil is incorrect or missing",
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for Error {}
