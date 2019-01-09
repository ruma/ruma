//! Crate **ruma_signatures** implements digital signatures according to the
//! [Matrix](https://matrix.org/) specification.
//!
//! Digital signatures are used by Matrix homeservers to verify the authenticity of events in the
//! Matrix system, as well as requests between homeservers for federation. Each homeserver has one
//! or more signing key pairs which it uses to sign all events and federation requests. Matrix
//! clients and other Matrix homeservers can ask the homeserver for its public keys and use those
//! keys to verify the signed data.
//!
//! Each signing key pair has an identifier, which consists of the name of the digital signature
//! algorithm it uses and a "version" string, separated by a colon. The version is an arbitrary
//! identifier used to distinguish key pairs using the same algorithm from the same homeserver.
//!
//! # Signing JSON
//!
//! A homeserver signs JSON with a key pair:
//!
//! ```rust,no_run
//! # extern crate ruma_signatures;
//! # extern crate serde_json;
//! # fn main() {
//! # use ruma_signatures::KeyPair;
//! # let public_key = [0; 32];
//! # let private_key = [0; 32];
//! // Create an Ed25519 key pair.
//! let key_pair = ruma_signatures::Ed25519KeyPair::new(
//!     &public_key, // &[u8]
//!     &private_key, // &[u8]
//!     "1".to_string(), // The "version" of the key.
//! ).expect("the provided keys should be suitable for Ed25519");
//! let value = serde_json::from_str("{}").expect("an empty JSON object should deserialize");
//! ruma_signatures::sign_json(&key_pair, &value).expect("value is a a JSON object"); // `Signature`
//! # }
//! ```
//!
//! # Signing Matrix events
//!
//! Signing an event uses a more involved process than signing arbitrary JSON.
//! Event signing is not yet implemented by ruma_signatures.
//!
//! # Verifying signatures
//!
//! A client application or another homeserver can verify a signature on arbitrary JSON:
//!
//! ```rust,no_run
//! # extern crate ruma_signatures;
//! # extern crate serde_json;
//! # fn main() {
//! # let public_key = [0; 32];
//! # let signature_bytes = [0, 32];
//! let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).expect(
//!     "key identifier should be valid"
//! );
//! let value = serde_json::from_str("{}").expect("an empty JSON object should deserialize");
//! let verifier = ruma_signatures::Ed25519Verifier::new();
//! assert!(ruma_signatures::verify_json(&verifier, &public_key, &signature, &value).is_ok());
//! # }
//! ```
//!
//! Verifying signatures of Matrix events is not yet implemented by ruma_signatures.
//!
//! # Signature sets
//!
//! Signatures that a homeserver has added to an event are stored in a JSON object under the
//! "signatures" key in the event's JSON representation:
//!
//! ```json
//! {
//!   "content": {},
//!   "event_type": "not.a.real.event",
//!   "signatures": {
//!     "example.com": {
//!       "ed25519:1": "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
//!     }
//!   }
//! }
//! ```
//!
//! The keys inside the "signatures" object are the hostnames of homeservers that have added
//! signatures. Within each of those objects are a set of signatures, keyed by the signing key
//! pair's identifier.
//!
//! This inner object can be created by serializing a `SignatureSet`:
//!
//! ```rust,no_run
//! # extern crate ruma_signatures;
//! # extern crate serde;
//! # extern crate serde_json;
//! # fn main() {
//! # let signature_bytes = [0, 32];
//! let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).expect(
//!     "key identifier should be valid"
//! );
//! let mut signature_set = ruma_signatures::SignatureSet::new();
//! signature_set.insert(signature);
//! serde_json::to_string(&signature_set).expect("signature_set should serialize");
//! # }
//! ```
//!
//! This code produces the object under the "example.com" key in the preceeding JSON. Similarly,
//! a `SignatureSet` can be produced by deserializing JSON that follows this form.
//!
//! The outer object (the map of server names to signature sets) is a `Signatures` value and
//! created like this:
//!
//! ```rust,no_run
//! # extern crate ruma_signatures;
//! # extern crate serde;
//! # extern crate serde_json;
//! # fn main() {
//! # let signature_bytes = [0, 32];
//! let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).expect(
//!     "key identifier should be valid"
//! );
//! let mut signature_set = ruma_signatures::SignatureSet::new();
//! signature_set.insert(signature);
//! let mut signatures = ruma_signatures::Signatures::new();
//! signatures.insert("example.com", signature_set).expect("example.com is a valid server name");
//! serde_json::to_string(&signatures).expect("signatures should serialize");
//! # }
//! ```
//!
//! Just like the `SignatureSet` itself, the `Signatures` value can also be deserialized from JSON.

#![deny(missing_docs, warnings)]

use std::{
    collections::{HashMap, HashSet},
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

use base64::{decode_config, encode_config, CharacterSet, Config};
use lazy_static::lazy_static;
use ring::signature::{verify, Ed25519KeyPair as RingEd25519KeyPair, ED25519};
use serde::{
    de::{Error as SerdeError, MapAccess, Unexpected, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::{to_string, Value};
use untrusted::Input;
use url::Url;

pub use url::Host;

lazy_static! {
    static ref BASE64_CONFIG: Config = Config::new(CharacterSet::Standard, false);
}

/// Signs an arbitrary JSON object.
///
/// # Parameters
///
/// * key_pair: A cryptographic key pair used to sign the JSON.
/// * value: A JSON object to be signed according to the Matrix specification.
///
/// # Errors
///
/// Returns an error if the JSON value is not a JSON object.
pub fn sign_json<K>(key_pair: &K, value: &Value) -> Result<Signature, Error>
where
    K: KeyPair,
{
    let json = to_canonical_json(value)?;

    Ok(key_pair.sign(json.as_bytes()))
}

/// Converts a JSON object into the "canonical" string form, suitable for signing.
///
/// # Parameters
///
/// * value: The `serde_json::Value` (JSON value) to convert.
///
/// # Errors
///
/// Returns an error if the provided JSON value is not a JSON object.
pub fn to_canonical_json(value: &Value) -> Result<String, Error> {
    if !value.is_object() {
        return Err(Error::new("JSON value must be a JSON object"));
    }

    let mut owned_value = value.clone();

    {
        let object = owned_value
            .as_object_mut()
            .expect("safe since we checked above");
        object.remove("signatures");
        object.remove("unsigned");
    }

    to_string(&owned_value).map_err(|error| Error::new(error.description()))
}

/// Use a public key to verify a signature of a JSON object.
///
/// # Parameters
///
/// * verifier: A `Verifier` appropriate for the digital signature algorithm that was used.
/// * public_key: The public key of the key pair used to sign the JSON, as a series of bytes.
/// * signature: The `Signature` to verify.
/// * value: The `serde_json::Value` (JSON value) that was signed.
///
/// # Errors
///
/// Returns an error if verification fails.
pub fn verify_json<V>(
    verifier: &V,
    public_key: &[u8],
    signature: &Signature,
    value: &Value,
) -> Result<(), Error>
where
    V: Verifier,
{
    verifier.verify_json(public_key, signature, to_canonical_json(value)?.as_bytes())
}

/// An error when trying to extract the algorithm and version from a key identifier.
#[derive(Debug)]
enum SplitError<'a> {
    InvalidLength(usize),
    UnknownAlgorithm(&'a str),
}

/// Extract the algorithm and version from a key identifier.
fn split_id(id: &str) -> Result<(Algorithm, String), SplitError<'_>> {
    const SIGNATURE_ID_LENGTH: usize = 2;

    let signature_id: Vec<&str> = id.split(':').collect();

    let signature_id_length = signature_id.len();

    if signature_id_length != SIGNATURE_ID_LENGTH {
        return Err(SplitError::InvalidLength(signature_id_length));
    }

    let algorithm_input = signature_id[0];

    let algorithm = match algorithm_input {
        "ed25519" => Algorithm::Ed25519,
        algorithm => return Err(SplitError::UnknownAlgorithm(algorithm)),
    };

    Ok((algorithm, signature_id[1].to_string()))
}

/// The algorithm used for signing data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    /// The Ed25519 digital signature algorithm.
    Ed25519,
}

/// An Ed25519 key pair.
pub struct Ed25519KeyPair {
    ring_key_pair: RingEd25519KeyPair,
    version: String,
}

/// A verifier for Ed25519 digital signatures.
#[derive(Clone, Copy, Debug)]
pub struct Ed25519Verifier;

/// An error produced when ruma_signatures operations fail.
#[derive(Clone, Debug)]
pub struct Error {
    message: String,
}

/// A cryptographic key pair for digitally signing data.
pub trait KeyPair: Sized {
    /// Initializes a new key pair.
    ///
    /// # Parameters
    ///
    /// * public_key: The public key of the key pair.
    /// * private_key: The private key of the key pair.
    /// * version: The "version" of the key used for this signature.
    ///   Versions are used as an identifier to distinguish signatures generated from different keys
    ///   but using the same algorithm on the same homeserver.
    ///
    /// # Errors
    ///
    /// Returns an error if the public and private keys provided are invalid for the implementing
    /// algorithm.
    fn new(public_key: &[u8], private_key: &[u8], version: String) -> Result<Self, Error>;

    /// Signs a JSON object.
    ///
    /// # Parameters
    ///
    /// * message: An arbitrary series of bytes to sign.
    fn sign(&self, message: &[u8]) -> Signature;
}

/// A digital signature.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Signature {
    algorithm: Algorithm,
    signature: Vec<u8>,
    version: String,
}

/// A map of server names to sets of digital signatures created by that server.
#[derive(Clone, Debug)]
pub struct Signatures {
    map: HashMap<Host, SignatureSet>,
}

/// Serde Visitor for deserializing `Signatures`.
struct SignaturesVisitor;

/// A set of digital signatures created by a single homeserver.
#[derive(Clone, Debug)]
pub struct SignatureSet {
    set: HashSet<Signature>,
}

/// Serde Visitor for deserializing `SignatureSet`.
struct SignatureSetVisitor;

/// A digital signature verifier.
pub trait Verifier {
    /// Use a public key to verify a signature against the JSON object that was signed.
    ///
    /// # Parameters
    ///
    /// * public_key: The public key of the key pair used to sign the message.
    /// * signature: The `Signature` to verify.
    /// * message: The message that was signed.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    fn verify_json(
        &self,
        public_key: &[u8],
        signature: &Signature,
        message: &[u8],
    ) -> Result<(), Error>;
}

impl KeyPair for Ed25519KeyPair {
    fn new(public_key: &[u8], private_key: &[u8], version: String) -> Result<Self, Error> {
        Ok(Ed25519KeyPair {
            ring_key_pair: RingEd25519KeyPair::from_seed_and_public_key(
                Input::from(private_key),
                Input::from(public_key),
            )
            .map_err(|_| Error::new("invalid key pair"))?,
            version: version,
        })
    }

    fn sign(&self, message: &[u8]) -> Signature {
        Signature {
            algorithm: Algorithm::Ed25519,
            signature: self.ring_key_pair.sign(message).as_ref().to_vec(),
            version: self.version.clone(),
        }
    }
}

impl Ed25519Verifier {
    /// Creates a new `Ed25519Verifier`.
    pub fn new() -> Self {
        Ed25519Verifier
    }
}

impl Verifier for Ed25519Verifier {
    fn verify_json(
        &self,
        public_key: &[u8],
        signature: &Signature,
        message: &[u8],
    ) -> Result<(), Error> {
        verify(
            &ED25519,
            Input::from(public_key),
            Input::from(message),
            Input::from(signature.as_bytes()),
        )
        .map_err(|_| Error::new("signature verification failed"))
    }
}

impl Error {
    /// Creates a new error.
    ///
    /// # Parameters
    ///
    /// * message: The error message.
    pub fn new<T>(message: T) -> Self
    where
        T: Into<String>,
    {
        Error {
            message: message.into(),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
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
            SplitError::InvalidLength(_) => Error::new("malformed signature ID"),
            SplitError::UnknownAlgorithm(algorithm) => {
                Error::new(format!("unknown algorithm: {}", algorithm))
            }
        })?;

        Ok(Signature {
            algorithm: algorithm,
            signature: bytes.to_vec(),
            version: version,
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
    pub fn base64(&self) -> String {
        encode_config(self.signature.as_slice(), *BASE64_CONFIG)
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

impl Signatures {
    /// Initializes a new empty Signatures.
    pub fn new() -> Self {
        Signatures {
            map: HashMap::new(),
        }
    }

    /// Initializes a new empty Signatures with room for a specific number of servers.
    ///
    /// # Parameters
    ///
    /// * capacity: The number of items to allocate memory for.
    pub fn with_capacity(capacity: usize) -> Self {
        Signatures {
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
        let url_string = format!("https://{}", server_name);
        let url = Url::parse(&url_string)
            .map_err(|_| Error::new(format!("invalid server name: {}", server_name)))?;

        let host = match url.host() {
            Some(host) => host.to_owned(),
            None => return Err(Error::new(format!("invalid server name: {}", server_name))),
        };

        Ok(self.map.insert(host, signature_set))
    }

    /// The number of servers in the collection.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl<'de> Deserialize<'de> for Signatures {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SignaturesVisitor)
    }
}

impl Serialize for Signatures {
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

impl<'de> Visitor<'de> for SignaturesVisitor {
    type Value = Signatures;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "digital signatures")
    }

    fn visit_map<M>(self, mut visitor: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut signatures = match visitor.size_hint() {
            Some(capacity) => Signatures::with_capacity(capacity),
            None => Signatures::new(),
        };

        while let Some((server_name, signature_set)) =
            visitor.next_entry::<String, SignatureSet>()?
        {
            if let Err(_) = signatures.insert(&server_name, signature_set) {
                return Err(M::Error::invalid_value(
                    Unexpected::Str(&server_name),
                    &self,
                ));
            }
        }

        Ok(signatures)
    }
}

impl SignatureSet {
    /// Initializes a new empty SignatureSet.
    pub fn new() -> Self {
        SignatureSet {
            set: HashSet::new(),
        }
    }

    /// Initializes a new empty SignatureSet with room for a specific number of signatures.
    ///
    /// # Parameters
    ///
    /// * capacity: The number of items to allocate memory for.
    pub fn with_capacity(capacity: usize) -> Self {
        SignatureSet {
            set: HashSet::with_capacity(capacity),
        }
    }

    /// Adds a signature to the set.
    ///
    /// The boolean return value indicates whether or not the value was actually inserted, since
    /// subsequent inserts of the same signature have no effect.
    ///
    /// # Parameters
    ///
    /// * signature: A `Signature` to insert into the set.
    pub fn insert(&mut self, signature: Signature) -> bool {
        self.set.insert(signature)
    }

    /// The number of signatures in the set.
    pub fn len(&self) -> usize {
        self.set.len()
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

impl Serialize for SignatureSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_serializer = serializer.serialize_map(Some(self.len()))?;

        for signature in self.set.iter() {
            map_serializer.serialize_key(&signature.id())?;
            map_serializer.serialize_value(&signature.base64())?;
        }

        map_serializer.end()
    }
}

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
                SplitError::UnknownAlgorithm(algorithm) => {
                    M::Error::invalid_value(Unexpected::Str(algorithm), &self)
                }
            })?;

            let signature_bytes: Vec<u8> = match decode_config(&value, *BASE64_CONFIG) {
                Ok(raw) => raw,
                Err(error) => return Err(M::Error::custom(error.description())),
            };

            let signature = Signature {
                algorithm: algorithm,
                signature: signature_bytes,
                version: version,
            };

            signature_set.insert(signature);
        }

        Ok(signature_set)
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = match *self {
            Algorithm::Ed25519 => "ed25519",
        };

        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod test {
    use base64::decode_config;
    use serde_json::{from_str, to_string, to_value};

    use super::{
        sign_json, verify_json, Ed25519KeyPair, Ed25519Verifier, KeyPair, Signature, SignatureSet,
        Signatures, BASE64_CONFIG,
    };

    const PUBLIC_KEY: &'static str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
    const PRIVATE_KEY: &'static str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";

    const EMPTY_JSON_SIGNATURE: &'static str =
        "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ";
    const MINIMAL_JSON_SIGNATURE: &'static str =
        "KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw";

    #[test]
    fn sign_empty_json() {
        let key_pair = Ed25519KeyPair::new(
            decode_config(&PUBLIC_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            decode_config(&PRIVATE_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            "1".to_string(),
        )
        .unwrap();

        let value = from_str("{}").unwrap();

        let signature = sign_json(&key_pair, &value).unwrap();

        assert_eq!(signature.base64(), EMPTY_JSON_SIGNATURE);
    }

    #[test]
    fn verify_empty_json() {
        let signature = Signature::new(
            "ed25519:1",
            decode_config(&EMPTY_JSON_SIGNATURE, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let value = from_str("{}").unwrap();

        let verifier = Ed25519Verifier::new();

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            &signature,
            &value,
        )
        .is_ok());
    }

    #[test]
    fn signatures_empty_json() {
        #[derive(serde_derive::Serialize)]
        struct EmptyWithSignatures {
            signatures: Signatures,
        }

        let signature = Signature::new(
            "ed25519:1",
            decode_config(&EMPTY_JSON_SIGNATURE, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let mut signature_set = SignatureSet::with_capacity(1);
        signature_set.insert(signature);

        let mut signatures = Signatures::with_capacity(1);
        signatures.insert("domain", signature_set).ok();

        let empty = EmptyWithSignatures {
            signatures: signatures,
        };

        let json = to_string(&empty).unwrap();

        assert_eq!(
            json,
            r#"{"signatures":{"domain":{"ed25519:1":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}}"#
        );
    }

    #[test]
    fn sign_minimal_json() {
        #[derive(serde_derive::Serialize)]
        struct Alpha {
            one: u8,
            two: String,
        }

        #[derive(serde_derive::Serialize)]
        struct ReverseAlpha {
            two: String,
            one: u8,
        }

        let key_pair = Ed25519KeyPair::new(
            decode_config(&PUBLIC_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            decode_config(&PRIVATE_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            "1".to_string(),
        )
        .unwrap();

        let alpha = Alpha {
            one: 1,
            two: "Two".to_string(),
        };

        let reverse_alpha = ReverseAlpha {
            two: "Two".to_string(),
            one: 1,
        };

        let alpha_value = to_value(alpha).expect("alpha should serialize");
        let alpha_signature = sign_json(&key_pair, &alpha_value).unwrap();

        assert_eq!(alpha_signature.base64(), MINIMAL_JSON_SIGNATURE);

        let reverse_alpha_value = to_value(reverse_alpha).expect("reverse_alpha should serialize");
        let reverse_alpha_signature = sign_json(&key_pair, &reverse_alpha_value).unwrap();

        assert_eq!(reverse_alpha_signature.base64(), MINIMAL_JSON_SIGNATURE);
    }

    #[test]
    fn verify_minimal_json() {
        let signature = Signature::new(
            "ed25519:1",
            decode_config(&MINIMAL_JSON_SIGNATURE, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let value = from_str(
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        ).unwrap();

        let verifier = Ed25519Verifier::new();

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            &signature,
            &value,
        )
        .is_ok());

        let reverse_value = from_str(
            r#"{"two":"Two","signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"one":1}"#
        ).unwrap();

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            &signature,
            &reverse_value,
        )
        .is_ok());
    }

    #[test]
    fn signatures_minimal_json() {
        #[derive(serde_derive::Serialize)]
        struct MinimalWithSignatures {
            one: u8,
            signatures: Signatures,
            two: String,
        }

        let signature = Signature::new(
            "ed25519:1",
            decode_config(&MINIMAL_JSON_SIGNATURE, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let mut signature_set = SignatureSet::with_capacity(1);
        signature_set.insert(signature);

        let mut signatures = Signatures::with_capacity(1);
        signatures.insert("domain", signature_set).ok();

        let minimal = MinimalWithSignatures {
            one: 1,
            signatures: signatures.clone(),
            two: "Two".to_string(),
        };

        let json = to_string(&minimal).unwrap();
        assert_eq!(
            json,
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        );
    }

    #[test]
    fn fail_verify() {
        let signature = Signature::new(
            "ed25519:1",
            decode_config(&EMPTY_JSON_SIGNATURE, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let value = from_str(r#"{"not":"empty"}"#).unwrap();

        let verifier = Ed25519Verifier::new();

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, *BASE64_CONFIG)
                .unwrap()
                .as_slice(),
            &signature,
            &value,
        )
        .is_err());
    }
}
