#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! Digital signatures according to the [Matrix](https://matrix.org/) specification.
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
//! In JSON representations, both signatures and hashes appear as base64-encoded strings, using the
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

#![warn(missing_docs)]

use ruma_common::serde::{AsRefStr, DisplayAsRefStr};

pub use self::{
    error::{Error, JsonError, ParseError, VerificationError},
    functions::{
        canonical_json, content_hash, hash_and_sign_event, reference_hash, sign_json, verify_event,
        verify_json,
    },
    keys::{Ed25519KeyPair, KeyPair, PublicKeyMap, PublicKeySet},
    signatures::Signature,
    verification::Verified,
};

mod error;
mod functions;
mod keys;
mod signatures;
mod verification;

/// The algorithm used for signing data.
#[derive(Clone, Debug, Eq, Hash, PartialEq, AsRefStr, DisplayAsRefStr)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum Algorithm {
    /// The Ed25519 digital signature algorithm.
    Ed25519,
}

/// Extract the algorithm and version from a key identifier.
fn split_id(id: &str) -> Result<(Algorithm, String), Error> {
    /// The length of a valid signature ID.
    const SIGNATURE_ID_LENGTH: usize = 2;

    let signature_id: Vec<&str> = id.split(':').collect();

    let signature_id_length = signature_id.len();

    if signature_id_length != SIGNATURE_ID_LENGTH {
        return Err(Error::InvalidLength(signature_id_length));
    }

    let version = signature_id[1];

    #[cfg(feature = "compat-signature-id")]
    const EXTRA_ALLOWED: [u8; 3] = [b'_', b'+', b'/'];
    #[cfg(not(feature = "compat-signature-id"))]
    const EXTRA_ALLOWED: [u8; 1] = [b'_'];
    if !version.bytes().all(|ch| ch.is_ascii_alphanumeric() || EXTRA_ALLOWED.contains(&ch)) {
        return Err(Error::InvalidVersion(version.into()));
    }

    let algorithm_input = signature_id[0];

    let algorithm = match algorithm_input {
        "ed25519" => Algorithm::Ed25519,
        algorithm => return Err(Error::UnsupportedAlgorithm(algorithm.into())),
    };

    Ok((algorithm, signature_id[1].to_owned()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pkcs8::{der::Decode, PrivateKeyInfo};
    use ruma_common::{
        serde::{base64::Standard, Base64},
        RoomVersionId,
    };
    use serde_json::{from_str as from_json_str, to_string as to_json_string};

    use super::{
        canonical_json, hash_and_sign_event, sign_json, verify_event, verify_json, Ed25519KeyPair,
    };

    fn pkcs8() -> Vec<u8> {
        const ENCODED: &str = "\
            MFECAQEwBQYDK2VwBCIEINjozvdfbsGEt6DD+7Uf4PiJ/YvTNXV2mIPc/\
            tA0T+6tgSEA3TPraTczVkDPTRaX4K+AfUuyx7Mzq1UafTXypnl0t2k\
        ";

        Base64::<Standard>::parse(ENCODED).unwrap().into_inner()
    }

    /// Convenience method for getting the public key as a string
    fn public_key_string() -> Base64 {
        Base64::new(PrivateKeyInfo::from_der(&pkcs8()).unwrap().public_key.unwrap().to_owned())
    }

    /// Convenience for converting a string of JSON into its canonical form.
    fn test_canonical_json(input: &str) -> String {
        let object = from_json_str(input).unwrap();
        canonical_json(&object).unwrap()
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

        assert_eq!(&test_canonical_json(r#"{"b":"2","a":"1"}"#), r#"{"a":"1","b":"2"}"#);

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
        let key_pair = Ed25519KeyPair::from_der(&pkcs8(), "1".into()).unwrap();

        let mut value = from_json_str("{}").unwrap();

        sign_json("domain", &key_pair, &mut value).unwrap();

        assert_eq!(
            to_json_string(&value).unwrap(),
            r#"{"signatures":{"domain":{"ed25519:1":"lXjsnvhVlz8t3etR+6AEJ0IT70WujeHC1CFjDDsVx0xSig1Bx7lvoi1x3j/2/GPNjQM4a2gD34UqsXFluaQEBA"}}}"#
        );
    }

    #[test]
    fn verify_empty_json() {
        let value = from_json_str(r#"{"signatures":{"domain":{"ed25519:1":"lXjsnvhVlz8t3etR+6AEJ0IT70WujeHC1CFjDDsVx0xSig1Bx7lvoi1x3j/2/GPNjQM4a2gD34UqsXFluaQEBA"}}}"#).unwrap();

        let mut signature_set = BTreeMap::new();
        signature_set.insert("ed25519:1".into(), public_key_string());

        let mut public_key_map = BTreeMap::new();
        public_key_map.insert("domain".into(), signature_set);

        verify_json(&public_key_map, &value).unwrap();
    }

    #[test]
    fn sign_minimal_json() {
        let key_pair = Ed25519KeyPair::from_der(&pkcs8(), "1".into()).unwrap();

        let mut alpha_object = from_json_str(r#"{ "one": 1, "two": "Two" }"#).unwrap();
        sign_json("domain", &key_pair, &mut alpha_object).unwrap();

        assert_eq!(
            to_json_string(&alpha_object).unwrap(),
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"t6Ehmh6XTDz7qNWI0QI5tNPSliWLPQP/+Fzz3LpdCS7q1k2G2/5b5Embs2j4uG3ZeivejrzqSVoBcdocRpa+AQ"}},"two":"Two"}"#
        );

        let mut reverse_alpha_object =
            from_json_str(r#"{ "two": "Two", "one": 1 }"#).expect("reverse_alpha should serialize");
        sign_json("domain", &key_pair, &mut reverse_alpha_object).unwrap();

        assert_eq!(
            to_json_string(&reverse_alpha_object).unwrap(),
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"t6Ehmh6XTDz7qNWI0QI5tNPSliWLPQP/+Fzz3LpdCS7q1k2G2/5b5Embs2j4uG3ZeivejrzqSVoBcdocRpa+AQ"}},"two":"Two"}"#
        );
    }

    #[test]
    fn verify_minimal_json() {
        let value = from_json_str(
            r#"{"one":1,"signatures":{"domain":{"ed25519:1":"t6Ehmh6XTDz7qNWI0QI5tNPSliWLPQP/+Fzz3LpdCS7q1k2G2/5b5Embs2j4uG3ZeivejrzqSVoBcdocRpa+AQ"}},"two":"Two"}"#
        ).unwrap();

        let mut signature_set = BTreeMap::new();
        signature_set.insert("ed25519:1".into(), public_key_string());

        let mut public_key_map = BTreeMap::new();
        public_key_map.insert("domain".into(), signature_set);

        verify_json(&public_key_map, &value).unwrap();

        let reverse_value = from_json_str(
            r#"{"two":"Two","signatures":{"domain":{"ed25519:1":"t6Ehmh6XTDz7qNWI0QI5tNPSliWLPQP/+Fzz3LpdCS7q1k2G2/5b5Embs2j4uG3ZeivejrzqSVoBcdocRpa+AQ"}},"one":1}"#
        ).unwrap();

        verify_json(&public_key_map, &reverse_value).unwrap();
    }

    #[test]
    fn fail_verify_json() {
        let value = from_json_str(r#"{"not":"empty","signatures":{"domain":"lXjsnvhVlz8t3etR+6AEJ0IT70WujeHC1CFjDDsVx0xSig1Bx7lvoi1x3j/2/GPNjQM4a2gD34UqsXFluaQEBA"}}"#).unwrap();

        let mut signature_set = BTreeMap::new();
        signature_set.insert("ed25519:1".into(), public_key_string());

        let mut public_key_map = BTreeMap::new();
        public_key_map.insert("domain".into(), signature_set);

        verify_json(&public_key_map, &value).unwrap_err();
    }

    #[test]
    fn sign_minimal_event() {
        let key_pair = Ed25519KeyPair::from_der(&pkcs8(), "1".into()).unwrap();

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

        let mut object = from_json_str(json).unwrap();
        hash_and_sign_event("domain", &key_pair, &mut object, &RoomVersionId::V5).unwrap();

        assert_eq!(
            to_json_string(&object).unwrap(),
            r#"{"auth_events":[],"content":{},"depth":3,"hashes":{"sha256":"5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"},"origin":"domain","origin_server_ts":1000000,"prev_events":[],"room_id":"!x:domain","sender":"@a:domain","signatures":{"domain":{"ed25519:1":"PxOFMn6ORll8PFSQp0IRF6037MEZt3Mfzu/ROiT/gb/ccs1G+f6Ddoswez4KntLPBI3GKCGIkhctiK37JOy2Aw"}},"type":"X","unsigned":{"age_ts":1000000}}"#
        );
    }

    #[test]
    fn sign_redacted_event() {
        let key_pair = Ed25519KeyPair::from_der(&pkcs8(), "1".into()).unwrap();

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

        let mut object = from_json_str(json).unwrap();
        hash_and_sign_event("domain", &key_pair, &mut object, &RoomVersionId::V5).unwrap();

        assert_eq!(
            to_json_string(&object).unwrap(),
            r#"{"content":{"body":"Here is the message content"},"event_id":"$0:domain","hashes":{"sha256":"onLKD1bGljeBWQhWZ1kaP9SorVmRQNdN5aM2JYU2n/g"},"origin":"domain","origin_server_ts":1000000,"room_id":"!r:domain","sender":"@u:domain","signatures":{"domain":{"ed25519:1":"D2V+qWBJssVuK/pEUJtwaYMdww2q1fP4PRCo226ChlLz8u8AWmQdLKes19NMjs/X0Hv0HIjU0c1TDKFMtGuoCA"}},"type":"m.room.message","unsigned":{"age_ts":1000000}}"#
        );
    }

    #[test]
    fn verify_minimal_event() {
        let mut signature_set = BTreeMap::new();
        signature_set.insert("ed25519:1".into(), public_key_string());

        let mut public_key_map = BTreeMap::new();
        public_key_map.insert("domain".into(), signature_set);

        let value = from_json_str(
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
                        "ed25519:1": "PxOFMn6ORll8PFSQp0IRF6037MEZt3Mfzu/ROiT/gb/ccs1G+f6Ddoswez4KntLPBI3GKCGIkhctiK37JOy2Aw"
                    }
                },
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#
        ).unwrap();

        verify_event(&public_key_map, &value, &RoomVersionId::V5).unwrap();
    }
}
