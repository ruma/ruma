//! Error conditions.

use std::str::Utf8Error;

/// An error encountered when trying to parse an invalid ID string.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The identifier or a required part of it is empty.
    #[error("identifier or required part of it is empty")]
    Empty,

    /// The identifier contains invalid characters.
    #[error("identifier contains invalid characters")]
    InvalidCharacters,

    /// The string isn't a valid Matrix ID.
    #[error("invalid matrix ID: {0}")]
    InvalidMatrixId(#[from] MatrixIdError),

    /// The string isn't a valid Matrix.to URI.
    #[error("invalid matrix.to URI: {0}")]
    InvalidMatrixToUri(#[from] MatrixToError),

    /// The string isn't a valid Matrix URI.
    #[error("invalid matrix URI: {0}")]
    InvalidMatrixUri(#[from] MatrixUriError),

    /// The mxc:// isn't a valid Matrix Content URI.
    #[error("invalid Matrix Content URI: {0}")]
    InvalidMxcUri(#[from] MxcUriError),

    /// The value isn't a valid VoIP version Id.
    #[error("invalid VoIP version ID: {0}")]
    InvalidVoipVersionId(#[from] VoipVersionIdError),

    /// The server name part of the the ID string is not a valid server name.
    #[error("server name is not a valid IP address or domain name")]
    InvalidServerName,

    /// The string isn't valid UTF-8.
    #[error("invalid UTF-8")]
    InvalidUtf8,

    /// The ID exceeds 255 bytes (or 32 codepoints for a room version ID).
    #[error("ID exceeds 255 bytes")]
    MaximumLengthExceeded,

    /// The ID is missing the colon delimiter between localpart and server name, or between key
    /// algorithm and key name / version.
    #[error("required colon is missing")]
    MissingColon,

    /// The ID is missing the correct leading sigil.
    #[error("leading sigil is incorrect or missing")]
    MissingLeadingSigil,
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Self::InvalidUtf8
    }
}

/// An error occurred while validating an MXC URI.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MxcUriError {
    /// MXC URI did not start with `mxc://`.
    #[error("MXC URI schema was not mxc://")]
    WrongSchema,

    /// MXC URI did not have first slash, required for `server.name/media_id`.
    #[error("MXC URI does not have first slash")]
    MissingSlash,

    /// Media identifier malformed due to invalid characters detected.
    ///
    /// Valid characters are (in regex notation) `[A-Za-z0-9_-]+`.
    /// See [here](https://spec.matrix.org/v1.10/client-server-api/#security-considerations-5) for more details.
    #[error("Media Identifier malformed, invalid characters")]
    MediaIdMalformed,

    /// Server identifier malformed: invalid IP or domain name.
    #[error("invalid Server Name")]
    ServerNameMalformed,
}

/// An error occurred while validating a `MatrixId`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MatrixIdError {
    /// The string contains an invalid number of parts.
    #[error("invalid number of parts")]
    InvalidPartsNumber,

    /// The string is missing a room ID or alias.
    #[error("missing room ID or alias")]
    MissingRoom,

    /// The string contains no identifier.
    #[error("no identifier")]
    NoIdentifier,

    /// The string contains too many identifiers.
    #[error("too many identifiers")]
    TooManyIdentifiers,

    /// The string contains an unknown identifier.
    #[error("unknown identifier")]
    UnknownIdentifier,

    /// The string contains two identifiers that cannot be paired.
    #[error("unknown identifier pair")]
    UnknownIdentifierPair,

    /// The string contains an unknown identifier type.
    #[error("unknown identifier type")]
    UnknownType,
}

/// An error occurred while validating a `matrix.to` URI.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MatrixToError {
    /// String is not a valid URI.
    #[error("given string is not a valid URL")]
    InvalidUrl,

    /// String did not start with `https://matrix.to/#/`.
    #[error("base URL is not https://matrix.to/#/")]
    WrongBaseUrl,

    /// String has an unknown additional argument.
    #[error("unknown additional argument")]
    UnknownArgument,
}

/// An error occurred while validating a `MatrixURI`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MatrixUriError {
    /// The string does not start with `matrix:`.
    #[error("scheme is not 'matrix:'")]
    WrongScheme,

    /// The string contains too many actions.
    #[error("too many actions")]
    TooManyActions,

    /// The string contains an unknown query item.
    #[error("unknown query item")]
    UnknownQueryItem,
}

/// An error occurred while validating a `VoipVersionId`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum VoipVersionIdError {
    /// The value of the `UInt` is not 0.
    #[error("UInt value is not 0")]
    WrongUintValue,
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::Error;

    #[test]
    fn small_error_type() {
        assert!(size_of::<Error>() <= 8);
    }
}
