use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// DOCS
// generic?
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Base64<B = Vec<u8>> {
    bytes: B,
}

// See https://github.com/matrix-org/matrix-doc/issues/3211
const BASE64_CONFIG: base64::Config = base64::STANDARD_NO_PAD.decode_allow_trailing_bits(true);

impl<B: AsRef<[u8]>> Base64<B> {
    /// Create a `Base64` instance from raw bytes, to be base64-encoded in serialialization.
    pub fn new(bytes: B) -> Self {
        Self { bytes }
    }

    /// Get the raw bytes held by this `Base64` instance.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Encode the bytes contained in this `Base64` instance to unpadded base64.
    pub fn encode(&self) -> String {
        base64::encode_config(&self.bytes, BASE64_CONFIG)
    }
}

impl Base64 {
    /// Create a `Base64` instance containing an empty `Vec<u8>`.
    pub fn empty() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Parse some base64-encoded data to create a `Base64` instance.
    pub fn parse(encoded: impl AsRef<[u8]>) -> Result<Self, base64::DecodeError> {
        base64::decode_config(encoded, BASE64_CONFIG).map(Self::new)
    }
}

impl<B: AsRef<[u8]>> fmt::Debug for Base64<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode().fmt(f)
    }
}

impl<B: AsRef<[u8]>> fmt::Display for Base64<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode().fmt(f)
    }
}

impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = crate::deserialize_cow_str(deserializer)?;
        Self::parse(&*encoded).map_err(de::Error::custom)
    }
}

impl<B: AsRef<[u8]>> Serialize for Base64<B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.encode())
    }
}
