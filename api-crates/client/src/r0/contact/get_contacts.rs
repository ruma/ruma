//! [GET /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-account-3pid)

use std::time::SystemTime;

use ruma_api::ruma_api;
use ruma_common::thirdparty::Medium;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get a list of 3rd party contacts associated with the user's account.",
        method: GET,
        name: "get_contacts",
        path: "/_matrix/client/r0/account/3pid",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of third party identifiers the homeserver has associated with the user's
        /// account.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub threepids: Vec<ThirdPartyIdentifier>,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given third party identifiers.
    pub fn new(threepids: Vec<ThirdPartyIdentifier>) -> Self {
        Self { threepids }
    }
}

/// An identifier external to Matrix.
///
/// To create an instance of this type, first create a `ThirdPartyIdentifierInit` and convert it to
/// this type using `ThirdPartyIdentifier::Init` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ThirdPartyIdentifier {
    /// The third party identifier address.
    pub address: String,

    /// The medium of third party identifier.
    pub medium: Medium,

    /// The time when the identifier was validated by the identity server.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub validated_at: SystemTime,

    /// The time when the homeserver associated the third party identifier with the user.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub added_at: SystemTime,
}

/// Initial set of fields of `ThirdPartyIdentifier`.
///
/// This struct will not be updated even if additional fields are added to `ThirdPartyIdentifier`
/// in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
pub struct ThirdPartyIdentifierInit {
    /// The third party identifier address.
    pub address: String,

    /// The medium of third party identifier.
    pub medium: Medium,

    /// The time when the identifier was validated by the identity server.
    pub validated_at: SystemTime,

    /// The time when the homeserver associated the third party identifier with the user.
    pub added_at: SystemTime,
}

impl From<ThirdPartyIdentifierInit> for ThirdPartyIdentifier {
    fn from(init: ThirdPartyIdentifierInit) -> Self {
        let ThirdPartyIdentifierInit { address, medium, validated_at, added_at } = init;
        ThirdPartyIdentifier { address, medium, validated_at, added_at }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{Medium, ThirdPartyIdentifier};

    #[test]
    fn third_party_identifier_serde() {
        let third_party_id = ThirdPartyIdentifier {
            address: "monkey@banana.island".into(),
            medium: Medium::Email,
            validated_at: UNIX_EPOCH + Duration::from_millis(1_535_176_800_000),
            added_at: UNIX_EPOCH + Duration::from_millis(1_535_336_848_756),
        };

        let third_party_id_serialized = json!({
            "medium": "email",
            "address": "monkey@banana.island",
            "validated_at": 1_535_176_800_000u64,
            "added_at": 1_535_336_848_756u64
        });

        assert_eq!(to_json_value(third_party_id.clone()).unwrap(), third_party_id_serialized);
        assert_eq!(third_party_id, from_json_value(third_party_id_serialized).unwrap());
    }
}
