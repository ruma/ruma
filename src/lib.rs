//! ruma-signatures provides functionality for creating digital signatures according to the
//! [Matrix](https://matrix.org/) specification.

#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;
extern crate serde_json;
extern crate sodiumoxide;

use std::collections::HashSet;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use rustc_serialize::base64::{CharacterSet, Config, Newline, ToBase64};
use serde_json::{Value, to_string};
use sodiumoxide::init;
use sodiumoxide::crypto::sign::{SecretKey, Signature as SodiumSignature, sign_detached};

lazy_static! {
    static ref _LIBSODIUM_INIT: bool = init();
}

static BASE64_CONFIG: Config = Config {
    char_set: CharacterSet::Standard,
    newline: Newline::CRLF,
    pad: false,
    line_length: None,
};

/// An error produced when signing data fails.
#[derive(Debug)]
pub struct Error {
    message: String,
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

/// A single digital signature.
///
/// Generated from `SigningKey`.
#[derive(Debug)]
pub struct Signature {
    algorithm: SigningAlgorithm,
    signature: SodiumSignature,
    version: String,
}

/// A set of signatures created by a single homeserver.
pub type SignatureSet = HashSet<Signature>;

/// The algorithm used for signing.
#[derive(Clone, Copy, Debug)]
pub enum SigningAlgorithm {
    /// The Ed25519 digital signature algorithm.
    Ed25519,
}

/// A signing key, consisting of an algorithm, a secret key, and a key version.
#[derive(Debug)]
pub struct SigningKey {
    algorithm: SigningAlgorithm,
    key: SecretKey,
    version: String,
}

impl Signature {
    /// The algorithm used to generate the signature.
    pub fn algorithm(&self) -> SigningAlgorithm {
        self.algorithm
    }

    /// The raw bytes of the signature.
    pub fn as_bytes(&self) -> &[u8] {
        self.signature.as_ref()
    }

    /// A Base64 encoding of the signature.
    pub fn base64(&self) -> String {
        self.signature.as_ref().to_base64(BASE64_CONFIG)
    }

    /// A string containing the signature algorithm and the key "version" separated by a colon.
    pub fn id(&self) -> String {
        format!("ed25519:{}", self.version())
    }

    /// The "version" of the key used for this signature.
    ///
    /// Versions are used as an identifier to distinguish signatures generated from different keys
    /// but using the same algorithm on the same homeserver.
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl SigningKey {
    /// Initialize a new signing key.
    ///
    /// # Parameters
    ///
    /// * algorithm: The digital signature algorithm to use.
    /// * key: A 64-byte secret key.
    /// * version: The "version" of the key used for this signature.
    ///   Versions are used as an identifier to distinguish signatures generated from different keys
    ///   but using the same algorithm on the same homeserver.
    pub fn new(algorithm: SigningAlgorithm, key: [u8; 64], version: String) -> Self {
        SigningKey {
            algorithm: algorithm,
            key: SecretKey(key),
            version: version,
        }
    }

    /// Sign a JSON object.
    pub fn sign(&self, value: &Value) -> Result<Signature, Error> {
        if !value.is_object() {
            return Err(Error::new("JSON value must be a JSON object"));
        }

        let mut owned_value = value.clone();

        {
            let mut hash = owned_value.as_object_mut().unwrap(); // Safe since we checked above.
            hash.remove("signatures");
            hash.remove("unsigned");
        }

        let json = match to_string(&owned_value) {
            Ok(json) => json,
            Err(error) => return Err(Error::new(error.description())),
        };

        Ok(Signature {
            algorithm: self.algorithm,
            signature: sign_detached(json.as_bytes(), &self.key),
            version: self.version.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use rustc_serialize::base64::FromBase64;
    use serde_json::from_str;
    use sodiumoxide::crypto::sign::{SecretKey, Seed, keypair_from_seed};

    use super::{SigningAlgorithm, SigningKey};

    #[test]
    fn empty_json() {
        let seed_vec = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA1".from_base64().unwrap();
        let seed = Seed::from_slice(&seed_vec[..]).unwrap();
        let (_pubkey, seckey) = keypair_from_seed(&seed);
        let SecretKey(raw_seckey) = seckey;
        let signing_key = SigningKey::new(SigningAlgorithm::Ed25519, raw_seckey, "1".to_owned());
        let value = from_str("{}").unwrap();
        let actual = signing_key.sign(&value).unwrap().base64();
        assert_eq!(
            actual,
            "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
        );
    }
}
