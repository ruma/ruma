//! Error conditions.

use std::fmt::{self, Display, Formatter};

/// An error encountered when trying to parse an invalid ID string.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// The ID's localpart contains invalid characters.
    ///
    /// Only relevant for user IDs.
    InvalidCharacters,
    /// The localpart of the ID string is not valid (because it is empty).
    InvalidLocalPart,
    /// The server name part of the the ID string is not a valid server name.
    InvalidServerName,
    /// The ID exceeds 255 bytes (or 32 codepoints for a room version ID.)
    MaximumLengthExceeded,
    /// The ID is less than 4 characters (or is an empty room version ID.)
    MinimumLengthNotSatisfied,
    /// The ID is missing the colon delimiter between localpart and server name.
    MissingDelimiter,
    /// The ID is missing the leading sigil.
    MissingSigil,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::InvalidCharacters => "localpart contains invalid characters",
            Error::InvalidLocalPart => "localpart is empty",
            Error::InvalidServerName => "server name is not a valid IP address or domain name",
            Error::MaximumLengthExceeded => "ID exceeds 255 bytes",
            Error::MinimumLengthNotSatisfied => "ID must be at least 4 characters",
            Error::MissingDelimiter => "colon is required between localpart and server name",
            Error::MissingSigil => "leading sigil is missing",
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for Error {}
