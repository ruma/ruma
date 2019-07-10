//! Digital signatures and collections of signatures.

use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error as _,
    fmt::{Formatter, Result as FmtResult},
};

use base64::{decode_config, encode_config, STANDARD_NO_PAD};
use serde::{
    de::{Error as SerdeError, MapAccess, Unexpected, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use url::{Host, Url};

use crate::{Algorithm, Error};

/// A digital signature.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Signature {
    /// The cryptographic algorithm to use.
    pub(crate) algorithm: Algorithm,

    /// The signature data.
    pub(crate) signature: Vec<u8>,

    /// The version of the signature.
    pub(crate) version: String,
}

impl Signature {
    /// Creates a signature from raw bytes.
    ///
    /// # Parameters
    ///
    /// * id: A key identifier, e.g. "ed25519:1".
    /// * bytes: The digital signature, as a series of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the key identifier is invalid.
    pub fn new(id: &str, bytes: &[u8]) -> Result<Self, Error> {
        let (algorithm, version) = split_id(id).map_err(|split_error| match split_error {
            SplitError::InvalidLength(length) => Error::new(format!("malformed signature ID: expected exactly 2 segment separated by a colon, found {}", length)),
            SplitError::InvalidVersion(version) => Error::new(format!("malformed signature ID: expected version to contain only characters in the character set `[a-zA-Z0-9_]`, found `{}`", version)),
            SplitError::UnknownAlgorithm(algorithm) => {
                Error::new(format!("unknown algorithm: {}", algorithm))
            }
        })?;

        Ok(Self {
            algorithm,
            signature: bytes.to_vec(),
            version,
        })
    }

    /// The algorithm used to generate the signature.
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// The raw bytes of the signature.
    pub fn as_bytes(&self) -> &[u8] {
        self.signature.as_slice()
    }

    /// A Base64 encoding of the signature.
    ///
    /// Uses the standard character set with no padding.
    pub fn base64(&self) -> String {
        encode_config(self.signature.as_slice(), STANDARD_NO_PAD)
    }

    /// The key identifier, a string containing the signature algorithm and the key "version"
    /// separated by a colon, e.g. "ed25519:1".
    pub fn id(&self) -> String {
        format!("{}:{}", self.algorithm, self.version)
    }

    /// The "version" of the key used for this signature.
    ///
    /// Versions are used as an identifier to distinguish signatures generated from different keys
    /// but using the same algorithm on the same homeserver.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// A map from server names to sets of digital signatures created by that server.
///
/// # Examples
///
/// Creating and serializing a `SignatureMap`:
///
/// ```rust
/// const SIGNATURE_BYTES: &str =
///     "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ";
///
/// // Create a `Signature` from the raw bytes of the signature.
/// let signature_bytes = base64::decode_config(&SIGNATURE_BYTES, base64::STANDARD_NO_PAD).unwrap();
/// let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).unwrap();
///
/// // Create a `SignatureSet` and insert the signature into it.
/// let mut signature_set = ruma_signatures::SignatureSet::new();
/// signature_set.insert(signature);
///
/// // Create a `SignatureMap` and insert the set into it, keyed by the homeserver name.
/// let mut signature_map = ruma_signatures::SignatureMap::new();
/// signature_map.insert("example.com", signature_set).unwrap();
///
/// // Serialize the map to JSON.
/// assert!(serde_json::to_string(&signature_map).is_ok());
/// ```
///
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignatureMap {
    /// A map from homeservers to sets of signatures for the homeserver.
    map: HashMap<Host, SignatureSet>,
}

impl SignatureMap {
    /// Initializes a new empty `SignatureMap`.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Initializes a new empty `SignatureMap` with room for a specific number of servers.
    ///
    /// # Parameters
    ///
    /// * capacity: The number of items to allocate memory for.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a signature set for a server.
    ///
    /// If no signature set for the given server existed in the collection, `None` is returned.
    /// Otherwise, the signature set is returned.
    ///
    /// # Parameters
    ///
    /// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
    /// * signature_set: The `SignatureSet` containing the digital signatures made by the server.
    ///
    /// # Errors
    ///
    /// Returns an error if the given server name cannot be parsed as a valid host.
    pub fn insert(
        &mut self,
        server_name: &str,
        signature_set: SignatureSet,
    ) -> Result<Option<SignatureSet>, Error> {
        let host = server_name_to_host(server_name)?;

        Ok(self.map.insert(host, signature_set))
    }

    /// Gets the given server's corresponding signature set for in-place manipulation.
    ///
    /// # Parameters
    ///
    /// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
    ///
    /// # Errors
    ///
    /// Returns an error if the given server name cannot be parsed as a valid host.
    pub fn entry(&mut self, server_name: &str) -> Result<Entry<Host, SignatureSet>, Error> {
        let host = server_name_to_host(server_name)?;

        Ok(self.map.entry(host))
    }

    /// Gets a reference to the signature set for the given server, if any.
    ///
    /// # Parameters
    ///
    /// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
    ///
    /// # Errors
    ///
    /// Returns an error if the given server name cannot be parsed as a valid host.
    pub fn get(&self, server_name: &str) -> Result<Option<&SignatureSet>, Error> {
        let host = server_name_to_host(server_name)?;

        Ok(self.map.get(&host))
    }

    /// Gets a mutable reference to the signature set for the given server, if any.
    ///
    /// # Parameters
    ///
    /// * server_name: The hostname or IP of the homeserver, e.g. `example.com`.
    ///
    /// # Errors
    ///
    /// Returns an error if the given server name cannot be parsed as a valid host.
    pub fn get_mut(&mut self, server_name: &str) -> Result<Option<&mut SignatureSet>, Error> {
        let host = server_name_to_host(server_name)?;

        Ok(self.map.get_mut(&host))
    }

    /// The number of servers in the collection.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether or not the collection of signatures is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Serialize for SignatureMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_serializer = serializer.serialize_map(Some(self.len()))?;

        for (host, signature_set) in self.map.iter() {
            map_serializer.serialize_key(&host.to_string())?;
            map_serializer.serialize_value(signature_set)?;
        }

        map_serializer.end()
    }
}

impl<'de> Deserialize<'de> for SignatureMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SignatureMapVisitor)
    }
}

/// Serde Visitor for deserializing `SignatureMap`.
struct SignatureMapVisitor;

impl<'de> Visitor<'de> for SignatureMapVisitor {
    type Value = SignatureMap;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "digital signatures")
    }

    fn visit_map<M>(self, mut visitor: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut signatures = match visitor.size_hint() {
            Some(capacity) => SignatureMap::with_capacity(capacity),
            None => SignatureMap::new(),
        };

        while let Some((server_name, signature_set)) =
            visitor.next_entry::<String, SignatureSet>()?
        {
            if signatures.insert(&server_name, signature_set).is_err() {
                return Err(M::Error::invalid_value(
                    Unexpected::Str(&server_name),
                    &self,
                ));
            }
        }

        Ok(signatures)
    }
}

/// A set of digital signatures created by a single homeserver.
///
/// # Examples
///
/// Creating and serializing a `SignatureSet`:
///
/// ```rust
/// const SIGNATURE_BYTES: &str =
///     "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ";
///
/// // Create a `Signature` from the raw bytes of the signature.
/// let signature_bytes = base64::decode_config(&SIGNATURE_BYTES, base64::STANDARD_NO_PAD).unwrap();
/// let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).unwrap();
///
/// // Create a `SignatureSet` and insert the signature into it.
/// let mut signature_set = ruma_signatures::SignatureSet::new();
/// signature_set.insert(signature);
///
/// // Serialize the set to JSON.
/// assert!(serde_json::to_string(&signature_set).is_ok());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignatureSet {
    /// A set of signatures for a homeserver.
    map: HashMap<String, Signature>,
}

impl SignatureSet {
    /// Initializes a new empty SignatureSet.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Initializes a new empty SignatureSet with room for a specific number of signatures.
    ///
    /// # Parameters
    ///
    /// * capacity: The number of items to allocate memory for.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a signature to the set.
    ///
    /// If no signature with the given key ID existed in the collection, `None` is returned.
    /// Otherwise, the signature is returned.
    ///
    /// # Parameters
    ///
    /// * signature: A `Signature` to insert into the set.
    pub fn insert(&mut self, signature: Signature) -> Option<Signature> {
        self.map.insert(signature.id(), signature)
    }

    /// Gets a reference to the signature with the given key ID, if any.
    ///
    /// # Parameters
    ///
    /// * key_id: The identifier of the public key (e.g. "ed25519:1") for the desired signature.
    pub fn get(&self, key_id: &str) -> Option<&Signature> {
        self.map.get(key_id)
    }

    /// Gets a mutable reference to the signature with the given ID, if any.
    ///
    /// # Parameters
    ///
    /// * key_id: The identifier of the public key (e.g. "ed25519:1") for the desired signature.
    pub fn get_mut(&mut self, key_id: &str) -> Option<&mut Signature> {
        self.map.get_mut(key_id)
    }

    /// The number of signatures in the set.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether or not the set of signatures is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Serialize for SignatureSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_serializer = serializer.serialize_map(Some(self.len()))?;

        for signature in self.map.values() {
            map_serializer.serialize_key(&signature.id())?;
            map_serializer.serialize_value(&signature.base64())?;
        }

        map_serializer.end()
    }
}

impl<'de> Deserialize<'de> for SignatureSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SignatureSetVisitor)
    }
}

/// Serde Visitor for deserializing `SignatureSet`.
struct SignatureSetVisitor;

impl<'de> Visitor<'de> for SignatureSetVisitor {
    type Value = SignatureSet;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "a set of digital signatures")
    }

    fn visit_map<M>(self, mut visitor: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut signature_set = match visitor.size_hint() {
            Some(capacity) => SignatureSet::with_capacity(capacity),
            None => SignatureSet::new(),
        };

        while let Some((key, value)) = visitor.next_entry::<String, String>()? {
            let (algorithm, version) = split_id(&key).map_err(|split_error| match split_error {
                SplitError::InvalidLength(length) => M::Error::invalid_length(length, &self),
                SplitError::InvalidVersion(version) => {
                    M::Error::invalid_value(Unexpected::Str(version), &self)
                }
                SplitError::UnknownAlgorithm(algorithm) => {
                    M::Error::invalid_value(Unexpected::Str(algorithm), &self)
                }
            })?;

            let signature_bytes: Vec<u8> = match decode_config(&value, STANDARD_NO_PAD) {
                Ok(raw) => raw,
                Err(error) => return Err(M::Error::custom(error.description())),
            };

            let signature = Signature {
                algorithm,
                signature: signature_bytes,
                version,
            };

            signature_set.insert(signature);
        }

        Ok(signature_set)
    }
}

/// An error when trying to extract the algorithm and version from a key identifier.
#[derive(Clone, Debug, PartialEq)]
enum SplitError<'a> {
    /// The signature's ID does not have exactly two components separated by a colon.
    InvalidLength(usize),
    /// The signature's ID contains invalid characters in its version.
    InvalidVersion(&'a str),
    /// The signature uses an unknown algorithm.
    UnknownAlgorithm(&'a str),
}

/// Extract the algorithm and version from a key identifier.
fn split_id(id: &str) -> Result<(Algorithm, String), SplitError<'_>> {
    /// The length of a valid signature ID.
    const SIGNATURE_ID_LENGTH: usize = 2;

    let signature_id: Vec<&str> = id.split(':').collect();

    let signature_id_length = signature_id.len();

    if signature_id_length != SIGNATURE_ID_LENGTH {
        return Err(SplitError::InvalidLength(signature_id_length));
    }

    let version = signature_id[1];

    let invalid_character_index = version.find(|ch| {
        !((ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || ch == '_')
    });

    if invalid_character_index.is_some() {
        return Err(SplitError::InvalidVersion(version));
    }

    let algorithm_input = signature_id[0];

    let algorithm = match algorithm_input {
        "ed25519" => Algorithm::Ed25519,
        algorithm => return Err(SplitError::UnknownAlgorithm(algorithm)),
    };

    Ok((algorithm, signature_id[1].to_string()))
}

/// Attempts to convert a server name as a string into a `url::Host`.
fn server_name_to_host(server_name: &str) -> Result<Host, Error> {
    let url_string = format!("https://{}", server_name);
    let url = Url::parse(&url_string)
        .map_err(|_| Error::new(format!("invalid server name: {}", server_name)))?;

    match url.host() {
        Some(host) => Ok(host.to_owned()),
        None => Err(Error::new(format!("invalid server name: {}", server_name))),
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;

    #[test]
    fn valid_key_id() {
        assert!(Signature::new("ed25519:abcdef", &[]).is_ok());
    }

    #[test]
    fn invalid_valid_key_id_length() {
        assert!(Signature::new("ed25519:abcdef:123456", &[]).is_err());
    }

    #[test]
    fn invalid_key_id_version() {
        assert!(Signature::new("ed25519:abc!def", &[]).is_err());
    }

    #[test]
    fn invalid_key_id_algorithm() {
        assert!(Signature::new("foobar:abcdef", &[]).is_err());
    }
}
