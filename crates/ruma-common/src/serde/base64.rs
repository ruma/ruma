//!Transparent base64 encoding / decoding as part of (de)serialization.

use std::{fmt, marker::PhantomData};

use base64::engine::fast_portable::{self, FastPortable, FastPortableConfig};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

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
    // See https://github.com/matrix-org/matrix-spec/issues/838
    const CONFIG: FastPortableConfig = fast_portable::NO_PAD.with_decode_allow_trailing_bits(true);
    const ENGINE: FastPortable = FastPortable::from(&C::CONF.0, Self::CONFIG);
}

impl<C: Base64Config, B: AsRef<[u8]>> Base64<C, B> {
    /// Create a `Base64` instance from raw bytes, to be base64-encoded in serialialization.
    pub fn new(bytes: B) -> Self {
        Self { bytes, _phantom_conf: PhantomData }
    }

    /// Get a reference to the raw bytes held by this `Base64` instance.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Encode the bytes contained in this `Base64` instance to unpadded base64.
    pub fn encode(&self) -> String {
        base64::encode_engine(self.as_bytes(), &Self::ENGINE)
    }
}

impl<C, B> Base64<C, B> {
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

    /// Parse some base64-encoded data to create a `Base64` instance.
    pub fn parse(encoded: impl AsRef<[u8]>) -> Result<Self, Base64DecodeError> {
        base64::decode_engine(encoded, &Self::ENGINE).map(Self::new).map_err(Base64DecodeError)
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

impl<'de, C: Base64Config> Deserialize<'de> for Base64<C> {
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

/// An error that occurred while decoding a base64 string.
#[derive(Clone)]
pub struct Base64DecodeError(base64::DecodeError);

impl fmt::Debug for Base64DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Base64DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Base64DecodeError {}
