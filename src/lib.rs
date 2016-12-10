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
//! ).unwrap();
//! let value = serde_json::from_str("{}").unwrap(); // An empty JSON object.
//! let signature = ruma_signatures::sign_json(&key_pair, &value).unwrap(); // `Signature`
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
//! # let signature = ruma_signatures::Signature::new("ed25519:1", &[0; 32]).unwrap();
//! let value = serde_json::from_str("{}").unwrap(); // The same empty JSON object.
//! assert!(signature.verify_json(&public_key, &value).is_ok());
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
//! # let signature = ruma_signatures::Signature::new("ed25519:1", &[0; 32]).unwrap();
//! let mut signature_set = ruma_signatures::SignatureSet::new();
//! signature_set.insert(signature);
//! let json = serde_json::to_string(&signature_set).unwrap();
//! # }
//! ```
//!
//! This code produces the object under the "example.com" key in the preceeding JSON. Similarly,
//! a `SignatureSet` can be produced by deserializing JSON that follows this form.

extern crate ring;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate untrusted;

use std::collections::HashSet;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use ring::signature::{ED25519, Ed25519KeyPair as RingEd25519KeyPair, verify};
use rustc_serialize::base64::{CharacterSet, Config, FromBase64, Newline, ToBase64};
use serde::{Deserialize, Deserializer, Error as SerdeError, Serialize, Serializer};
use serde::de::{MapVisitor, Visitor};
use serde_json::{Value, to_string};
use untrusted::Input;

static BASE64_CONFIG: Config = Config {
    char_set: CharacterSet::Standard,
    newline: Newline::CRLF,
    pad: false,
    line_length: None,
};

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
pub fn sign_json<K>(key_pair: &K, value: &Value) -> Result<Signature, Error> where K: KeyPair {
    let json = to_canonical_json(value)?;

    Ok(key_pair.sign(json.as_bytes()))
}

/// Converts a JSON object into the "canonical" form, suitable for signing.
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
        let mut object = owned_value.as_object_mut().unwrap(); // Safe since we checked above.
        object.remove("signatures");
        object.remove("unsigned");
    }

    to_string(&owned_value).map_err(|error| Error::new(error.description()))
}

enum SplitError<'a> {
    InvalidLength(usize),
    UnknownAlgorithm(&'a str),
}

fn split_id(id: &str) -> Result<(Algorithm, String), SplitError> {
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

/// An Ed25519 key pair.
pub struct Ed25519KeyPair {
    ring_key_pair: RingEd25519KeyPair,
    version: String,
}

/// An error produced during signing or verification.
#[derive(Debug)]
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
    /// * message: An arbitrary binary value to sign.
    fn sign(&self, message: &[u8]) -> Signature;
}

/// A single digital signature.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Signature {
    algorithm: Algorithm,
    signature: Vec<u8>,
    version: String,
}

/// A set of signatures created by a single homeserver.
pub struct SignatureSet {
    set: HashSet<Signature>,
}

/// Serde Visitor for deserializing `SignatureSet`.
struct SignatureSetVisitor;

/// The algorithm used for signing data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    /// The Ed25519 digital signature algorithm.
    Ed25519,
}

impl KeyPair for Ed25519KeyPair {
    fn new(public_key: &[u8], private_key: &[u8], version: String) -> Result<Self, Error> {
        Ok(Ed25519KeyPair {
            ring_key_pair: RingEd25519KeyPair::from_bytes(
                private_key,
                public_key,
            ).map_err(|_| Error::new("invalid key pair"))?,
            version: version,
        })
    }

    fn sign(&self, message: &[u8]) -> Signature {
        Signature {
            algorithm: Algorithm::Ed25519,
            signature: self.ring_key_pair.sign(message).as_slice().to_vec(),
            version: self.version.clone(),
        }
    }
}

impl Error {
    pub fn new<T>(message: T) -> Self where T: Into<String> {
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
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl Signature {
    /// Creates a signature from raw bytes.
    pub fn new(id: &str, bytes: &[u8]) -> Result<Self, Error> {
        let (algorithm, version) = split_id(id).map_err(|split_error| {
            match split_error {
                SplitError::InvalidLength(_) => Error::new("malformed signature ID"),
                SplitError::UnknownAlgorithm(algorithm) => {
                    Error::new(format!("unknown algorithm: {}", algorithm))
                }
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
        self.signature.as_slice().to_base64(BASE64_CONFIG)
    }

    /// A string containing the signature algorithm and the key "version" separated by a colon.
    pub fn id(&self) -> String {
        format!("{}:{}", self.algorithm, self.version)
    }

    /// Use the public key to verify the signature against the JSON object that was signed.
    pub fn verify_json(&self, public_key: &[u8], value: &Value) -> Result<(), Error> {
        match self.algorithm {
            Algorithm::Ed25519 => {
                verify(
                    &ED25519,
                    Input::from(public_key),
                    Input::from(to_canonical_json(value)?.as_bytes()),
                    Input::from(self.as_bytes()),
                ).map_err(|_| Error::new("signature verification failed"))
            }
        }
    }

    /// The "version" of the key used for this signature.
    ///
    /// Versions are used as an identifier to distinguish signatures generated from different keys
    /// but using the same algorithm on the same homeserver.
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl SignatureSet {
    /// Initialize a new empty SignatureSet.
    pub fn new() -> Self {
        SignatureSet {
            set: HashSet::new(),
        }
    }

    /// Initialize a new empty SignatureSet with room for a specific number of signatures.
    pub fn with_capacity(capacity: usize) -> Self {
        SignatureSet {
            set: HashSet::with_capacity(capacity),
        }
    }

    /// Adds a signature to the set.
    ///
    /// The boolean return value indicates whether or not the value was actually inserted, since
    /// subsequent inserts of the same signature have no effect.
    pub fn insert(&mut self, signature: Signature) -> bool {
        self.set.insert(signature)
    }

    /// The number of signatures in the set.
    pub fn len(&self) -> usize {
        self.set.len()
    }
}

impl Deserialize for SignatureSet {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize_map(SignatureSetVisitor)
    }
}

impl Serialize for SignatureSet {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        let mut state = try!(serializer.serialize_map(Some(self.len())));

        for signature in self.set.iter() {
            try!(serializer.serialize_map_key(&mut state, signature.id()));
            try!(serializer.serialize_map_value(&mut state, signature.base64()));
        }

        serializer.serialize_map_end(state)
    }
}

impl Visitor for SignatureSetVisitor {
    type Value = SignatureSet;

    fn visit_map<M>(&mut self, mut visitor: M) -> Result<Self::Value, M::Error>
    where M: MapVisitor {
        let mut signature_set = SignatureSet::with_capacity(visitor.size_hint().0);

        while let Some((key, value)) = try!(visitor.visit::<String, String>()) {
            let (algorithm, version) = split_id(&key).map_err(|split_error| {
                match split_error {
                    SplitError::InvalidLength(length) => M::Error::invalid_length(length),
                    SplitError::UnknownAlgorithm(algorithm) => M::Error::invalid_value(algorithm),
                }
            })?;

            let signature_bytes: Vec<u8> = match value.from_base64() {
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

        try!(visitor.end());

        Ok(signature_set)
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let name = match *self {
            Algorithm::Ed25519 => "ed25519",
        };

        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod test {
    use rustc_serialize::base64::FromBase64;
    use serde_json::from_str;

    use super::{Ed25519KeyPair, KeyPair, Signature, sign_json};

    const PUBLIC_KEY: &'static str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
    const PRIVATE_KEY: &'static str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";

    const EMPTY_JSON_SIGNATURE: &'static str =
        "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ";

    #[test]
    fn sign_empty_json() {
        let key_pair = Ed25519KeyPair::new(
            &PUBLIC_KEY.from_base64().unwrap(),
            &PRIVATE_KEY.from_base64().unwrap(),
            "1".to_string(),
        ).unwrap();
        let value = from_str("{}").unwrap();
        let signature = sign_json(&key_pair, &value).unwrap();
        assert_eq!(signature.base64(), EMPTY_JSON_SIGNATURE);
    }

    #[test]
    fn verify_empty_json() {
        let signature = Signature::new(
            "ed25519:1",
            &EMPTY_JSON_SIGNATURE.from_base64().unwrap(),
        ).unwrap();
        let value = from_str("{}").unwrap();
        assert!(signature.verify_json(&PUBLIC_KEY.from_base64().unwrap(), &value).is_ok());
    }
}
