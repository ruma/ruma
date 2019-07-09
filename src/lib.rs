//! Crate `ruma_signatures` implements digital signatures according to the
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
//! # use ruma_signatures::{self, KeyPair};
//! # use serde_json;
//! # let public_key = [0; 32];
//! # let private_key = [0; 32];
//! // Create an Ed25519 key pair.
//! let key_pair = ruma_signatures::Ed25519KeyPair::new(
//!     &public_key, // &[u8]
//!     &private_key, // &[u8]
//!     "1".to_string(), // The "version" of the key.
//! ).expect("the provided keys should be suitable for Ed25519");
//! let value = serde_json::from_str("{}").expect("an empty JSON object should deserialize");
//! let signature = ruma_signatures::sign_json(
//!     &key_pair,
//!     &value,
//! ).expect("`value` must be a JSON object");
//! ```
//!
//! # Signing Matrix events
//!
//! Signing an event uses a more involved process than signing arbitrary JSON.
//! Event signing is not yet implemented by ruma-signatures.
//!
//! # Verifying signatures
//!
//! A client application or another homeserver can verify a signature on arbitrary JSON:
//!
//! ```rust,no_run
//! # use ruma_signatures;
//! # use serde_json;
//! # let public_key = [0; 32];
//! # let signature_bytes = [0, 32];
//! let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).expect(
//!     "key identifier should be valid"
//! );
//! let value = serde_json::from_str("{}").expect("an empty JSON object should deserialize");
//! let verifier = ruma_signatures::Ed25519Verifier;
//! assert!(ruma_signatures::verify_json(&verifier, &public_key, &signature, &value).is_ok());
//! ```
//!
//! Verifying signatures of Matrix events is not yet implemented by ruma-signatures.
//!
//! # Signature sets and maps
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
//! # use ruma_signatures;
//! # use serde;
//! # use serde_json;
//! # let signature_bytes = [0, 32];
//! let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).expect(
//!     "key identifier should be valid"
//! );
//! let mut signature_set = ruma_signatures::SignatureSet::new();
//! signature_set.insert(signature);
//! serde_json::to_string(&signature_set).expect("signature_set should serialize");
//! ```
//!
//! This code produces the object under the "example.com" key in the preceeding JSON. Similarly,
//! a `SignatureSet` can be produced by deserializing JSON that follows this form.
//!
//! The outer object (the map of server names to signature sets) is a `SignatureMap` value and
//! created like this:
//!
//! ```rust,no_run
//! # use ruma_signatures;
//! # use serde;
//! # use serde_json;
//! # let signature_bytes = [0, 32];
//! let signature = ruma_signatures::Signature::new("ed25519:1", &signature_bytes).expect(
//!     "key identifier should be valid"
//! );
//! let mut signature_set = ruma_signatures::SignatureSet::new();
//! signature_set.insert(signature);
//! let mut signature_map = ruma_signatures::SignatureMap::new();
//! signature_map.insert("example.com", signature_set).expect("example.com is a valid server name");
//! serde_json::to_string(&signature_map).expect("signature_map should serialize");
//! ```
//!
//! Just like the `SignatureSet` itself, the `SignatureMap` value can also be deserialized from
//! JSON.

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

pub use url::Host;

pub use functions::{
    content_hash, hash_and_sign_event, reference_hash, sign_json, to_canonical_json, verify_json,
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
    use base64::{decode_config, STANDARD_NO_PAD};
    use serde::Serialize;
    use serde_json::{from_str, to_string, to_value, Value};

    use super::{
        sign_json, to_canonical_json, verify_json, Ed25519KeyPair, Ed25519Verifier, KeyPair,
        Signature, SignatureMap, SignatureSet,
    };

    const PUBLIC_KEY: &str = "XGX0JRS2Af3be3knz2fBiRbApjm2Dh61gXDJA8kcJNI";
    const PRIVATE_KEY: &str = "YJDBA9Xnr2sVqXD9Vj7XVUnmFZcZrlw8Md7kMW+3XA0";

    const EMPTY_JSON_SIGNATURE: &str =
        "K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ";
    const MINIMAL_JSON_SIGNATURE: &str =
        "KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw";

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

        let value = from_str("{}").unwrap();

        let signature = sign_json(&key_pair, &value).unwrap();

        assert_eq!(signature.base64(), EMPTY_JSON_SIGNATURE);
    }

    #[test]
    fn verify_empty_json() {
        let signature = Signature::new(
            "ed25519:1",
            decode_config(&EMPTY_JSON_SIGNATURE, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let value = from_str("{}").unwrap();

        let verifier = Ed25519Verifier;

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            &signature,
            &value,
        )
        .is_ok());
    }

    #[test]
    fn signature_map_empty_json() {
        #[derive(Serialize)]
        struct EmptyWithSignatureMap {
            signatures: SignatureMap,
        }

        let signature = Signature::new(
            "ed25519:1",
            decode_config(&EMPTY_JSON_SIGNATURE, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let mut signature_set = SignatureSet::with_capacity(1);
        signature_set.insert(signature);

        let mut signature_map = SignatureMap::with_capacity(1);
        signature_map.insert("domain", signature_set).ok();

        let empty = EmptyWithSignatureMap {
            signatures: signature_map,
        };

        let json = to_string(&empty).unwrap();

        assert_eq!(
            json,
            r#"{"signatures":{"domain":{"ed25519:1":"K8280/U9SSy9IVtjBuVeLr+HpOB4BQFWbg+UZaADMtTdGYI7Geitb76LTrr5QV/7Xg4ahLwYGYZzuHGZKM5ZAQ"}}}"#
        );
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
            decode_config(&MINIMAL_JSON_SIGNATURE, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let value = from_str(
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"KqmLSbO39/Bzb0QIYE82zqLwsA+PDzYIpIRA2sRQ4sL53+sN6/fpNSoqE7BP7vBZhG6kYdD13EIMJpvhJI+6Bw"}},"two":"Two"}"#
        ).unwrap();

        let verifier = Ed25519Verifier;

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
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
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            &signature,
            &reverse_value,
        )
        .is_ok());
    }

    #[test]
    fn signature_map_minimal_json() {
        #[derive(Serialize)]
        struct MinimalWithSignatureMap {
            one: u8,
            signatures: SignatureMap,
            two: String,
        }

        let signature = Signature::new(
            "ed25519:1",
            decode_config(&MINIMAL_JSON_SIGNATURE, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let mut signature_set = SignatureSet::with_capacity(1);
        signature_set.insert(signature);

        let mut signature_map = SignatureMap::with_capacity(1);
        signature_map.insert("domain", signature_set).ok();

        let minimal = MinimalWithSignatureMap {
            one: 1,
            signatures: signature_map.clone(),
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
            decode_config(&EMPTY_JSON_SIGNATURE, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        let value = from_str(r#"{"not":"empty"}"#).unwrap();

        let verifier = Ed25519Verifier;

        assert!(verify_json(
            &verifier,
            decode_config(&PUBLIC_KEY, STANDARD_NO_PAD)
                .unwrap()
                .as_slice(),
            &signature,
            &value,
        )
        .is_err());
    }
}
