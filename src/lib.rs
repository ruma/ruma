//! Crate `ruma_signatures` implements digital signatures according to the
//! [Matrix](https://matrix.org/) specification.
//!
//! Digital signatures are used by Matrix homeservers to verify the authenticity of events in the
//! Matrix system, as well as requests between homeservers for federation. Each homeserver has one
//! or more signing key pairs (sometimes referred to as "verify keys") which it uses to sign all
//! events and federation requests. Matrix clients and other Matrix homeservers can ask the
//! homeserver for its public keys and use those keys to verify the signed data.
//!
//! Each signing key pair has an identifier, which consists of the name of the digital signature
//! algorithm it uses and a "version" string, separated by a colon. The version is an arbitrary
//! identifier used to distinguish key pairs using the same algorithm from the same homeserver.
//!
//! Arbitrary JSON objects can be signed as well as JSON representations of Matrix events. In both
//! cases, the signatures are stored within the JSON object itself under a `signatures` key. Events
//! are also required to contain hashes of their content, which are similarly stored within the
//! hashed JSON object under a `hashes` key.
//!
//! In JSON representations, both signatures and hashes appear as Base64-encoded strings, using the
//! standard character set, without padding.
//!
//! # Signing and hashing
//!
//! To sign an arbitrary JSON object, use the `sign_json` function. See the documentation of this
//! function for more details and a full example of use.
//!
//! Signing an event uses a more complicated process than signing arbitrary JSON, because events can
//! be redacted, and signatures need to remain valid even if data is removed from an event later.
//! Homeservers are required to generate hashes of event contents as well as signing events before
//! exchanging them with other homeservers. Although the algorithm for hashing and signing an event
//! is more complicated than for signing arbitrary JSON, the interface to a user of ruma-signatures
//! is the same. To hash and sign an event, use the `hash_and_sign_event` function. See the
//! documentation of this function for more details and a full example of use.
//!
//! # Verifying signatures and hashes
//!
//! When a homeserver receives data from another homeserver via the federation, it's necessary to
//! verify the authenticity and integrity of the data by verifying their signatures.
//!
//! To verify a signature on arbitrary JSON, use the `verify_json` function. To verify the
//! signatures and hashes on an event, use the `verify_event` function. See the documentation for
//! these respective functions for more details and full examples of use.

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub use functions::{
    canonical_json, content_hash, hash_and_sign_event, redact, reference_hash, sign_json,
    verify_event, verify_json,
};
pub use keys::{Ed25519KeyPair, KeyPair, PublicKeyMap, PublicKeySet};
pub use signatures::Signature;

mod functions;
mod keys;
mod signatures;
mod verification;

/// An error produced when ruma-signatures operations fail.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    /// A human-readable description of the error.
    message: String,
}

impl Error {
    /// Creates a new error.
    ///
    /// # Parameters
    ///
    /// * message: The error message.
    pub(crate) fn new<T>(message: T) -> Self
    where
        T: Into<String>,
    {
        Self {
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

impl From<base64::DecodeError> for Error {
    fn from(error: base64::DecodeError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

/// The algorithm used for signing data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    /// The Ed25519 digital signature algorithm.
    Ed25519,
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = match *self {
            Self::Ed25519 => "ed25519",
        };

        write!(f, "{}", name)
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use base64::{decode_config, STANDARD_NO_PAD};
    use serde_json::{from_str, json, to_string, to_value, Value};

    use super::{
        canonical_json, hash_and_sign_event, sign_json, verify_event, verify_json, Ed25519KeyPair,
    };

    const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
    const PRIVATE_KEY: &str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";

    /// Convenience for converting a string of JSON into its canonical form.
    fn test_canonical_json(input: &str) -> String {
        let value = from_str::<Value>(input).unwrap();

        canonical_json(&value).unwrap()
    }

    #[test]
    fn canonical_json_examples() {
        assert_eq!(&test_canonical_json("{}"), "{}");

        assert_eq!(
            &test_canonical_json(
                r#"{
                    "one": 1,
                    "two": "Two"
                }"#
            ),
            r#"{"one":1,"two":"Two"}"#
        );

        assert_eq!(
            &test_canonical_json(
                r#"{
                    "b": "2",
                    "a": "1"
                }"#
            ),
            r#"{"a":"1","b":"2"}"#
        );

        assert_eq!(
            &test_canonical_json(r#"{"b":"2","a":"1"}"#),
            r#"{"a":"1","b":"2"}"#
        );

        assert_eq!(
            &test_canonical_json(
                r#"{
                    "auth": {
                        "success": true,
                        "mxid": "@john.doe:example.com",
                        "profile": {
                            "display_name": "John Doe",
                            "three_pids": [
                                {
                                    "medium": "email",
                                    "address": "john.doe@example.org"
                                },
                                {
                                    "medium": "msisdn",
                                    "address": "123456789"
                                }
                            ]
                        }
                    }
                }"#
            ),
            r#"{"auth":{"mxid":"@john.doe:example.com","profile":{"display_name":"John Doe","three_pids":[{"address":"john.doe@example.org","medium":"email"},{"address":"123456789","medium":"msisdn"}]},"success":true}}"#
        );

        assert_eq!(
            &test_canonical_json(
                r#"{
                    "a": "日本語"
                }"#
            ),
            r#"{"a":"日本語"}"#
        );

        assert_eq!(
            &test_canonical_json(
                r#"{
                    "本": 2,
                    "日": 1
                }"#
            ),
            r#"{"日":1,"本":2}"#
        );

        assert_eq!(
            &test_canonical_json(
                r#"{
                    "a": "\u65E5"
                }"#
            ),
            r#"{"a":"日"}"#
        );

        assert_eq!(
            &test_canonical_json(
                r#"{
                "a": null
            }"#
            ),
            r#"{"a":null}"#
        );
    }

    #[test]
    fn sign_empty_json() {
        let key_pair = Ed25519KeyPair::new(
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            decode_config(&PRIVATE_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            "1".to_string(),
        )
        .unwrap();

        let mut value = from_str("{}").unwrap();

        sign_json("domain", &key_pair, &mut value).unwrap();

        assert_eq!(
            to_string(&value).unwrap(),
            r#"{"signatures":{"domain":{"ed25519:1":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}}"#
        );
    }

    #[test]
    fn verify_empty_json() {
        let value = from_str(r#"{"signatures":{"domain":{"ed25519:1":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}}"#).unwrap();

        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut public_key_map = HashMap::new();
        public_key_map.insert("domain".to_string(), signature_set);

        assert!(verify_json(&public_key_map, &value).is_ok());
    }

    #[test]
    fn sign_minimal_json() {
        let key_pair = Ed25519KeyPair::new(
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            decode_config(&PRIVATE_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            "1".to_string(),
        )
        .unwrap();

        let alpha = json!({
            "one": 1,
            "two": "Two",
        });

        let reverse_alpha = json!({
            "two": "Two",
            "one": 1,
        });

        let mut alpha_value = to_value(alpha).expect("alpha should serialize");
        sign_json("domain", &key_pair, &mut alpha_value).unwrap();

        assert_eq!(
            to_string(&alpha_value).unwrap(),
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        );

        let mut reverse_alpha_value =
            to_value(reverse_alpha).expect("reverse_alpha should serialize");
        sign_json("domain", &key_pair, &mut reverse_alpha_value).unwrap();

        assert_eq!(
            to_string(&reverse_alpha_value).unwrap(),
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        );
    }

    #[test]
    fn verify_minimal_json() {
        let value = from_str(
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        ).unwrap();

        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut public_key_map = HashMap::new();
        public_key_map.insert("domain".to_string(), signature_set);

        assert!(verify_json(&public_key_map, &value).is_ok());

        let reverse_value = from_str(
            r#"{"two":"Two","signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"one":1}"#
        ).unwrap();

        assert!(verify_json(&public_key_map, &reverse_value).is_ok());
    }

    #[test]
    fn fail_verify_json() {
        let value = from_str(r#"{"not":"empty","signatures":{"domain":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}"#).unwrap();

        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut public_key_map = HashMap::new();
        public_key_map.insert("domain".to_string(), signature_set);

        assert!(verify_json(&public_key_map, &value).is_err());
    }

    #[test]
    fn sign_minimal_event() {
        let key_pair = Ed25519KeyPair::new(
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            decode_config(&PRIVATE_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            "1".to_string(),
        )
        .unwrap();

        let json = r#"{
            "room_id": "!x:domain",
            "sender": "@a:domain",
            "origin": "domain",
            "origin_server_ts": 1000000,
            "signatures": {},
            "hashes": {},
            "type": "X",
            "content": {},
            "prev_events": [],
            "auth_events": [],
            "depth": 3,
            "unsigned": {
                "age_ts": 1000000
            }
        }"#;

        let mut value = from_str::<Value>(json).unwrap();
        hash_and_sign_event("domain", &key_pair, &mut value).unwrap();

        assert_eq!(
            to_string(&value).unwrap(),
            r#"{"auth_events":[],"content":{},"depth":3,"hashes":{"sha256":"5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"},"origin":"domain","origin_server_ts":1000000,"prev_events":[],"room_id":"!x:domain","sender":"@a:domain","signatures":{"domain":{"ed25519:1":"KxwGjPSDEtvnFgU00fwFz+l6d2pJM6XBIaMEn81SXPTRl16AqLAYqfIReFGZlHi5KLjAWbOoMszkwsQma+lYAg"}},"type":"X","unsigned":{"age_ts":1000000}}"#
        );
    }

    #[test]
    fn sign_redacted_event() {
        let key_pair = Ed25519KeyPair::new(
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            decode_config(&PRIVATE_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            "1".to_string(),
        )
        .unwrap();

        let json = r#"{
            "content": {
                "body": "Here is the message content"
            },
            "event_id": "$0:domain",
            "origin": "domain",
            "origin_server_ts": 1000000,
            "type": "m.room.message",
            "room_id": "!r:domain",
            "sender": "@u:domain",
            "signatures": {},
            "unsigned": {
                "age_ts": 1000000
            }
        }"#;

        let mut value = from_str::<Value>(json).unwrap();
        hash_and_sign_event("domain", &key_pair, &mut value).unwrap();

        assert_eq!(
            to_string(&value).unwrap(),
            r#"{"content":{"body":"Here is the message content"},"event_id":"$0:domain","hashes":{"sha256":"onLKD1bGljeBWQhWZ1kaP9SorVmRQNdN5aM2JYU2n/g"},"origin":"domain","origin_server_ts":1000000,"room_id":"!r:domain","sender":"@u:domain","signatures":{"domain":{"ed25519:1":"Wm+VzmOUOz08Ds+0NTWb1d4CZrVsJSikkeRxh6aCcUwu6pNC78FunoD7KNWzqFn241eYHYMGCA5McEiVPdhzBA"}},"type":"m.room.message","unsigned":{"age_ts":1000000}}"#
        );
    }

    #[test]
    fn verify_minimal_event() {
        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut public_key_map = HashMap::new();
        public_key_map.insert("domain".to_string(), signature_set);

        let value = from_str(
            r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@a:domain",
                "signatures": {
                    "domain": {
                        "ed25519:1": "KxwGjPSDEtvnFgU00fwFz+l6d2pJM6XBIaMEn81SXPTRl16AqLAYqfIReFGZlHi5KLjAWbOoMszkwsQma+lYAg"
                    }
                },
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#
        ).unwrap();

        assert!(verify_event(&public_key_map, &value).is_ok());
    }
}
