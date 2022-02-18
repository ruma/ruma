//! Error conditions.

use std::str::Utf8Error;

/// An error encountered when trying to parse an invalid ID string.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The client secret is empty.
    #[error("client secret is empty")]
    EmptyClientSecret,

    /// The room name is empty.
    #[error("room name is empty")]
    EmptyRoomName,

    /// The room version ID is empty.
    #[error("room version ID is empty")]
    EmptyRoomVersionId,

    /// The ID's localpart contains invalid characters.
    ///
    /// Only relevant for user IDs.
    #[error("localpart contains invalid characters")]
    InvalidCharacters,

    /// The key algorithm is invalid (e.g. empty).
    #[error("invalid key algorithm specified")]
    InvalidKeyAlgorithm,

    /// The key version contains outside of [a-zA-Z0-9_].
    #[error("key ID version contains invalid characters")]
    InvalidKeyVersion,

    /// The string isn't a valid Matrix ID.
    #[error("invalid matrix ID: {0}")]
    InvalidMatrixId(#[from] MatrixIdError),

    /// The string isn't a valid Matrix.to URI.
    #[error("invalid matrix.to URI: {0}")]
    InvalidMatrixToRef(#[from] MatrixToError),

    /// The mxc:// isn't a valid Matrix Content URI.
    #[error("invalid Matrix Content URI: {0}")]
    InvalidMxcUri(#[from] MxcUriError),

    /// The server name part of the the ID string is not a valid server name.
    #[error("server name is not a valid IP address or domain name")]
    InvalidServerName,

    /// The string isn't a valid URI.
    #[error("invalid URI")]
    InvalidUri,

    /// The string isn't valid UTF-8.
    #[error("invalid UTF-8")]
    InvalidUtf8,

    /// The ID exceeds 255 bytes (or 32 codepoints for a room version ID).
    #[error("ID exceeds 255 bytes")]
    MaximumLengthExceeded,

    /// The ID is missing the colon delimiter between localpart and server name, or between key
    /// algorithm and key name / version.
    #[error("required colon is missing")]
    MissingDelimiter,

    /// The ID is missing the correct leading sigil.
    #[error("leading sigil is incorrect or missing")]
    MissingLeadingSigil,
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Self::InvalidUtf8
    }
}

impl From<url::ParseError> for Error {
    fn from(_: url::ParseError) -> Self {
        Self::InvalidUri
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
    /// See [here](https://matrix.org/docs/spec/client_server/r0.6.1#id408) for more details.
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
}

/// An error occurred while validating an MXC URI.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MatrixToError {
    /// String did not start with `https://matrix.to/#/`.
    #[error("base URL is not https://matrix.to/#/")]
    WrongBaseUrl,

    /// String has an unknown additional argument.
    #[error("unknown additional argument")]
    UnknownArgument,
}
