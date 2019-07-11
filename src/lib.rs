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
//! # Signatures, maps, and sets
//!
//! An individual signature is represented in ruma-signatures by the `Signature` type. This type
//! encapsulates the raw bytes of the signature, the identifier for the signing key used, and the
//! algorithm used to create the signature.
//!
//! As mentioned, signatures that a homeserver has added to an event are stored in a JSON object
//! under the `signatures` key in the event's JSON representation:
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
//! The value of the the `signatures` key is represented in ruma-signatures by the `SignatureMap`
//! type. This type maps the name of a homeserver to a set of its signatures for the containing
//! data. The set of signatures for each homeserver (which appears as a map from key ID to signature
//! in the JSON representation) is represented in ruma-signatures by the `SignatureSet` type. Both
//! `SignatureMap`s and `SignatureSet`s can be serialized and deserialized with
//! [https://serde.rs/](Serde).
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
    missing_docs,
    warnings
)]
#![warn(
    clippy::empty_line_after_outer_attr,
    clippy::expl_impl_clone_on_copy,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::mem_forget,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::single_match_else,
    clippy::unicode_not_nfc,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::wrong_pub_self_convention,
    clippy::wrong_self_convention
)]

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub use functions::{
    content_hash, hash_and_sign_event, reference_hash, sign_json, to_canonical_json, verify_event,
    verify_json,
};
pub use keys::{Ed25519KeyPair, KeyPair};
pub use signatures::{Signature, SignatureMap, SignatureSet};
pub use verification::{Ed25519Verifier, Verifier};

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
            Algorithm::Ed25519 => "ed25519",
        };

        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use base64::{decode_config, STANDARD_NO_PAD};
    use serde::Serialize;
    use serde_json::{from_str, to_string, to_value, Value};

    use super::{
        hash_and_sign_event, sign_json, to_canonical_json, verify_event, verify_json,
        Ed25519KeyPair, Ed25519Verifier,
    };

    const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
    const PRIVATE_KEY: &str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";

    /// Convenience for converting a string of JSON into its canonical form.
    fn test_canonical_json(input: &str) -> String {
        let value = from_str::<Value>(input).unwrap();

        to_canonical_json(&value).unwrap()
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

        assert_eq!(&test_canonical_json(
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
            }"#),
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

        sign_json("example.com", &key_pair, &mut value).unwrap();

        assert_eq!(
            to_string(&value).unwrap(),
            r#"{"signatures":{"example.com":{"ed25519:1":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}}"#
        );
    }

    #[test]
    fn verify_empty_json() {
        let value = from_str(r#"{"signatures":{"example.com":{"ed25519:1":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}}"#).unwrap();

        let verifier = Ed25519Verifier;

        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut verify_key_map = HashMap::new();
        verify_key_map.insert("example.com".to_string(), signature_set);

        assert!(verify_json(&verifier, &verify_key_map, &value).is_ok());
    }

    #[test]
    fn sign_minimal_json() {
        #[derive(Serialize)]
        struct Alpha {
            one: u8,
            two: String,
        }

        #[derive(Serialize)]
        struct ReverseAlpha {
            two: String,
            one: u8,
        }

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

        let alpha = Alpha {
            one: 1,
            two: "Two".to_string(),
        };

        let reverse_alpha = ReverseAlpha {
            two: "Two".to_string(),
            one: 1,
        };

        let mut alpha_value = to_value(alpha).expect("alpha should serialize");
        sign_json("example.com", &key_pair, &mut alpha_value).unwrap();

        assert_eq!(
            to_string(&alpha_value).unwrap(),
            r#"{"one":1,"signatures":{"example.com":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        );

        let mut reverse_alpha_value =
            to_value(reverse_alpha).expect("reverse_alpha should serialize");
        sign_json("example.com", &key_pair, &mut reverse_alpha_value).unwrap();

        assert_eq!(
            to_string(&reverse_alpha_value).unwrap(),
            r#"{"one":1,"signatures":{"example.com":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        );
    }

    #[test]
    fn verify_minimal_json() {
        let value = from_str(
            r#"{"one":1,"signatures":{"example.com":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        ).unwrap();

        let verifier = Ed25519Verifier;

        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut verify_key_map = HashMap::new();
        verify_key_map.insert("example.com".to_string(), signature_set);

        assert!(verify_json(&verifier, &verify_key_map, &value).is_ok());

        let reverse_value = from_str(
            r#"{"two":"Two","signatures":{"example.com":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"one":1}"#
        ).unwrap();

        assert!(verify_json(&verifier, &verify_key_map, &reverse_value).is_ok());
    }

    #[test]
    fn fail_verify_json() {
        let value = from_str(r#"{"not":"empty","signatures":{"example.com":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}"#).unwrap();

        let verifier = Ed25519Verifier;

        let mut signature_set = HashMap::new();
        signature_set.insert("ed25519:1".to_string(), PUBLIC_KEY.to_string());

        let mut verify_key_map = HashMap::new();
        verify_key_map.insert("example.com".to_string(), signature_set);

        assert!(verify_json(&verifier, &verify_key_map, &value).is_err());
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

        let mut verify_key_map = HashMap::new();
        verify_key_map.insert("domain".to_string(), signature_set);

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

        let verifier = Ed25519Verifier;

        assert!(verify_event(&verifier, &verify_key_map, &value).is_ok());
    }
}
