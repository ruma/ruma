//! ruma-signatures provides functionality for creating digital signatures according to the
//! [Matrix](https://matrix.org/) specification.

#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;
extern crate serde_json;
extern crate sodiumoxide;

use std::error::Error;
use std::fmt::Display;

use rustc_serialize::base64::{CharacterSet, Config, Newline, ToBase64};
use serde_json::{Value, to_string};
use sodiumoxide::init;
use sodiumoxide::crypto::sign::{SecretKey, Signature, sign_detached};

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
pub struct SigningError {
    message: String,
}

impl SigningError {
    pub fn new<T>(message: T) -> Self where T: Into<String> {
        SigningError {
            message: message.into(),
        }
    }
}

impl Error for SigningError {
    fn description(&self) -> &str {
        self.message.as_ref()
    }
}

impl Display for SigningError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// The algorithm used for signing.
#[derive(Debug)]
pub enum SigningAlgorithm {
    Ed25519,
}

/// A signing key, consisting of an algoritm, a secret key, and a key version.
#[derive(Debug)]
pub struct SigningKey {
    algorithm: SigningAlgorithm,
    key: SecretKey,
    version: String,
}

impl SigningKey {
    /// Initialize a new signing key.
    pub fn new(algorithm: SigningAlgorithm, key: [u8; 64], version: String) -> Self {
        SigningKey {
            algorithm: algorithm,
            key: SecretKey(key),
            version: version,
        }
    }

    /// Sign a JSON object.
    pub fn sign(&self, value: &Value) -> Result<Signature, SigningError> {
        if !value.is_object() {
            return Err(SigningError::new("JSON value must be a JSON object"));
        }

        let mut owned_value = value.clone();

        {
            let mut hash = owned_value.as_object_mut().unwrap(); // Safe since we checked above.
            hash.remove("signatures");
            hash.remove("unsigned");
        }

        let json = match to_string(&owned_value) {
            Ok(json) => json,
            Err(error) => return Err(SigningError::new(error.description())),
        };

        Ok(sign_detached(json.as_bytes(), &self.key))
    }

    /// Sign and Base64 encode a JSON object.
    pub fn sign_and_base64_encode(&self, value: &Value) -> Result<String, SigningError> {
        let signature = try!(self.sign(value));

        Ok(signature.as_ref().to_base64(BASE64_CONFIG))
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
        let actual = signing_key.sign_and_base64_encode(&value).unwrap();
        assert_eq!(
            actual,
            "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"
        );
    }
}
