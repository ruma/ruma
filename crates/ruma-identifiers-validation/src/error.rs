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
    /// See [here](https://spec.matrix.org/v1.2/client-server-api/#security-considerations-5) for more details.
    #[error("Media Identifier malformed, invalid characters")]
    MediaIdMalformed,

    /// Server identifier malformed: invalid IP or domain name.
    #[error("invalid Server Name")]
    ServerNameMalformed,
}
