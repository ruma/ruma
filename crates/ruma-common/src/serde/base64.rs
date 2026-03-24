//! Transparent base64 encoding / decoding as part of (de)serialization.

use std::{fmt, marker::PhantomData};

use base64::{
    Engine,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig, general_purpose},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use zeroize::Zeroize;

/// A wrapper around `B` (usually `Vec<u8>`) that (de)serializes from / to a base64 string.
///
/// The base64 character set (and miscellaneous other encoding / decoding options) can be customized
/// through the generic parameter `C`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Base64<C = Standard, B = Vec<u8>> {
    bytes: B,
    // Invariant PhantomData, Send + Sync
    _phantom_conf: PhantomData<fn(C) -> C>,
}

impl<C, B> Zeroize for Base64<C, B>
where
    B: Zeroize,
{
    fn zeroize(&mut self) {
        self.bytes.zeroize();
    }
}

/// Config used for the [`Base64`] type.
pub trait Base64Config {
    /// The config as a constant.
    ///
    /// Opaque so our interface is not tied to the base64 crate version.
    #[doc(hidden)]
    const CONF: Conf;
}

#[doc(hidden)]
pub struct Conf(base64::alphabet::Alphabet);

/// Standard base64 character set without padding.
///
/// Allows trailing bits in decoding for maximum compatibility.
#[non_exhaustive]
// Easier than implementing these all for Base64 manually to avoid the `C: Trait` bounds.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Standard;

impl Base64Config for Standard {
    const CONF: Conf = Conf(base64::alphabet::STANDARD);
}

/// Url-safe base64 character set without padding.
///
/// Allows trailing bits in decoding for maximum compatibility.
#[non_exhaustive]
// Easier than implementing these all for Base64 manually to avoid the `C: Trait` bounds.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UrlSafe;

impl Base64Config for UrlSafe {
    const CONF: Conf = Conf(base64::alphabet::URL_SAFE);
}

impl<C: Base64Config, B> Base64<C, B> {
    const CONFIG: GeneralPurposeConfig = general_purpose::NO_PAD
        // See https://github.com/matrix-org/matrix-spec/issues/838
        .with_decode_allow_trailing_bits(true)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent);
    const ENGINE: GeneralPurpose = GeneralPurpose::new(&C::CONF.0, Self::CONFIG);
}

impl<C: Base64Config, B: AsRef<[u8]>> Base64<C, B> {
    /// Create a `Base64` instance from raw bytes, to be base64-encoded in serialization.
    pub fn new(bytes: B) -> Self {
        Self { bytes, _phantom_conf: PhantomData }
    }

    /// Get a reference to the raw bytes held by this `Base64` instance.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Encode the bytes contained in this `Base64` instance to unpadded base64.
    pub fn encode(&self) -> String {
        Self::ENGINE.encode(self.as_bytes())
    }
}

impl<C, B> Base64<C, B> {
    /// Get a reference to the raw bytes held by this `Base64` instance.
    pub fn as_inner(&self) -> &B {
        &self.bytes
    }

    /// Get the raw bytes held by this `Base64` instance.
    pub fn into_inner(self) -> B {
        self.bytes
    }
}

impl<C: Base64Config> Base64<C> {
    /// Create a `Base64` instance containing an empty `Vec<u8>`.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }
}

impl<C: Base64Config, B: TryFromBase64DecodedBytes> Base64<C, B> {
    /// Parse some base64-encoded data to create a `Base64` instance.
    pub fn parse(encoded: impl AsRef<[u8]>) -> Result<Self, Base64DecodeError> {
        let decoded = Self::ENGINE.decode(encoded).map_err(Base64DecodeError::base64)?;
        B::try_from_bytes(decoded).map(Self::new)
    }
}

impl<C: Base64Config, B: AsRef<[u8]>> fmt::Debug for Base64<C, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode().fmt(f)
    }
}

impl<C: Base64Config, B: AsRef<[u8]>> fmt::Display for Base64<C, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode().fmt(f)
    }
}

impl<'de, C: Base64Config, B: TryFromBase64DecodedBytes> Deserialize<'de> for Base64<C, B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = super::deserialize_cow_str(deserializer)?;
        Self::parse(&*encoded).map_err(de::Error::custom)
    }
}

impl<C: Base64Config, B: AsRef<[u8]>> Serialize for Base64<C, B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.encode())
    }
}

/// Marker trait for indicating which inner `B` "bytes" type can be converted from decoded base64
/// bytes.
///
/// This is used as a bound in [`Base64::parse()`] to provide a more helpful error message than
/// using a `TryFrom<Vec<u8>>` implementation.
pub trait TryFromBase64DecodedBytes: Sized + AsRef<[u8]> {
    /// Convert the given bytes to this type.
    #[doc(hidden)]
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Base64DecodeError>;
}

impl TryFromBase64DecodedBytes for Vec<u8> {
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Base64DecodeError> {
        Ok(bytes)
    }
}

impl<const N: usize> TryFromBase64DecodedBytes for [u8; N] {
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Base64DecodeError> {
        Self::try_from(bytes)
            .map_err(|bytes| Base64DecodeError::invalid_decoded_length(bytes.len(), N))
    }
}

/// An error that occurred while decoding a base64 string.
#[derive(Clone)]
pub struct Base64DecodeError(Base64DecodeErrorInner);

impl Base64DecodeError {
    /// Construct a `Base64DecodeError` from an invalid base64 encoding error.
    fn base64(error: base64::DecodeError) -> Self {
        Self(Base64DecodeErrorInner::Base64(error))
    }

    /// Construct a `Base64DecodeError` from an invalid decoded bytes length error.
    fn invalid_decoded_length(len: usize, expected: usize) -> Self {
        Self(Base64DecodeErrorInner::InvalidDecodedLength { len, expected })
    }
}

impl fmt::Debug for Base64DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Base64DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Base64DecodeErrorInner::Base64(error) => write!(f, "invalid base64 encoding: {error}"),
            Base64DecodeErrorInner::InvalidDecodedLength { len, expected } => {
                write!(f, "invalid decoded base64 bytes length: {len}, expected {expected}")
            }
        }
    }
}

impl std::error::Error for Base64DecodeError {}

/// An error that occurred while decoding a base64 string.
#[derive(Debug, Clone)]
enum Base64DecodeErrorInner {
    /// The base64 encoding is invalid.
    Base64(base64::DecodeError),

    /// The decoded bytes have the wrong length to fit into an array of fixed length.
    InvalidDecodedLength {
        /// The length of the input.
        len: usize,
        /// The expected length.
        expected: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::{Base64, Standard};

    #[test]
    fn parse_base64() {
        const INPUT: &str = "3UmJnEIzUr2xWyaUnJg5fXwRybwG5FVC6Gq\
            MHverEUn0ztuIsvVxX89JXX2pvdTsOBbLQx+4TVL02l4Cp5wPCm";
        const INPUT_WITH_PADDING: &str = "im9+knCkMNQNh9o6sbdcZw==";

        Base64::<Standard>::parse(INPUT).unwrap();
        Base64::<Standard>::parse(INPUT_WITH_PADDING)
            .expect("We should be able to decode padded Base64");

        // Check that we can parse with the correct length.
        Base64::<Standard, [u8; 32]>::parse(INPUT).unwrap_err();
        Base64::<Standard, [u8; 64]>::parse(INPUT).unwrap();
        Base64::<Standard, [u8; 32]>::parse(INPUT_WITH_PADDING).unwrap_err();
        Base64::<Standard, [u8; 16]>::parse(INPUT_WITH_PADDING).unwrap();
    }
}
