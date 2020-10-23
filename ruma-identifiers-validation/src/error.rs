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

    /// The key version contains outside of [a-zA-Z0-9_].
    InvalidKeyVersion,

    /// The server name part of the the ID string is not a valid server name.
    InvalidServerName,

    /// The ID exceeds 255 bytes (or 32 codepoints for a room version ID).
    MaximumLengthExceeded,

    /// The ID is missing the colon delimiter between localpart and server name.
    MissingDelimiter,

    /// The ID is missing the colon delimiter between key algorithm and device ID.
    MissingDeviceKeyDelimiter,

    /// The ID is missing the colon delimiter between key algorithm and version.
    MissingServerKeyDelimiter,

    /// The ID is missing the correct leading sigil.
    MissingSigil,

    /// The key algorithm is not recognized.
    UnknownKeyAlgorithm,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::EmptyRoomVersionId => "room version ID is empty",
            Error::InvalidCharacters => "localpart contains invalid characters",
            Error::InvalidKeyVersion => "key ID version contains invalid characters",
            Error::InvalidServerName => "server name is not a valid IP address or domain name",
            Error::MaximumLengthExceeded => "ID exceeds 255 bytes",
            Error::MissingDelimiter => "colon is required between localpart and server name",
            Error::MissingDeviceKeyDelimiter => "colon is required between algorithm and device ID",
            Error::MissingServerKeyDelimiter => "colon is required between algorithm and version",
            Error::MissingSigil => "leading sigil is incorrect or missing",
            Error::UnknownKeyAlgorithm => "unknown key algorithm specified",
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for Error {}
