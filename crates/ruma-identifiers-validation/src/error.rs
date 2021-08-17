//! Error conditions.

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

    /// The mxc:// isn't a valid Matrix Content URI.
    #[error("invalid Matrix Content URI: {0}")]
    InvalidMxcUri(#[from] MxcUriError),

    /// The server name part of the the ID string is not a valid server name.
    #[error("server name is not a valid IP address or domain name")]
    InvalidServerName,

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

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
pub enum MxcUriError {
    #[error("MXC URI schema was not mxc://")]
    WrongSchema,
    #[error("MXC URI does not have first slash")]
    MissingSlash,
    #[error("Media Identifier malformed, invalid characters")]
    MediaIdMalformed,
    #[error("invalid Server Name")]
    ServerNameMalformed,
}
