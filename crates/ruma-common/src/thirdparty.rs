//! Common types for the [third party networks module][thirdparty].
//!
//! [thirdparty]: https://matrix.org/docs/spec/client_server/r0.6.1#id153

use std::collections::BTreeMap;

use ruma_identifiers::{RoomAliasId, UserId};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::MilliSecondsSinceUnixEpoch;

/// Metadata about a third party protocol.
///
/// To create an instance of this type, first create a `ProtocolInit` and convert it via
/// `Protocol::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Protocol {
    /// Fields which may be used to identify a third party user.
    pub user_fields: Vec<String>,

    /// Fields which may be used to identify a third party location.
    pub location_fields: Vec<String>,

    /// A content URI representing an icon for the third party protocol.
    ///
    /// If you activate the `compat` feature, this field being absent in JSON will give you an
    /// empty string here.
    #[cfg_attr(feature = "compat", serde(default))]
    pub icon: String,

    /// The type definitions for the fields defined in `user_fields` and `location_fields`.
    pub field_types: BTreeMap<String, FieldType>,

    /// A list of objects representing independent instances of configuration.
    pub instances: Vec<ProtocolInstance>,
}

/// Initial set of fields of `Protocol`.
///
/// This struct will not be updated even if additional fields are added to `Prococol` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct ProtocolInit {
    /// Fields which may be used to identify a third party user.
    pub user_fields: Vec<String>,

    /// Fields which may be used to identify a third party location.
    pub location_fields: Vec<String>,

    /// A content URI representing an icon for the third party protocol.
    pub icon: String,

    /// The type definitions for the fields defined in `user_fields` and `location_fields`.
    pub field_types: BTreeMap<String, FieldType>,

    /// A list of objects representing independent instances of configuration.
    pub instances: Vec<ProtocolInstance>,
}

impl From<ProtocolInit> for Protocol {
    fn from(init: ProtocolInit) -> Self {
        let ProtocolInit { user_fields, location_fields, icon, field_types, instances } = init;
        Self { user_fields, location_fields, icon, field_types, instances }
    }
}

/// Metadata about an instance of a third party protocol.
///
/// To create an instance of this type, first create a `ProtocolInstanceInit` and convert it via
/// `ProtocolInstance::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ProtocolInstance {
    /// A human-readable description for the protocol, such as the name.
    pub desc: String,

    /// An optional content URI representing the protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Preset values for `fields` the client may use to search by.
    pub fields: BTreeMap<String, String>,

    /// A unique identifier across all instances.
    pub network_id: String,

    /// A unique identifier across all instances.
    #[cfg(feature = "unstable-pre-spec")]
    pub instance_id: String,
}

/// Initial set of fields of `Protocol`.
///
/// This struct will not be updated even if additional fields are added to `Prococol` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct ProtocolInstanceInit {
    /// A human-readable description for the protocol, such as the name.
    pub desc: String,

    /// Preset values for `fields` the client may use to search by.
    pub fields: BTreeMap<String, String>,

    /// A unique identifier across all instances.
    pub network_id: String,

    /// A unique identifier across all instances.
    #[cfg(feature = "unstable-pre-spec")]
    pub instance_id: String,
}

impl From<ProtocolInstanceInit> for ProtocolInstance {
    fn from(init: ProtocolInstanceInit) -> Self {
        let ProtocolInstanceInit {
            desc,
            fields,
            network_id,
            #[cfg(feature = "unstable-pre-spec")]
            instance_id,
        } = init;
        Self {
            desc,
            icon: None,
            fields,
            network_id,
            #[cfg(feature = "unstable-pre-spec")]
            instance_id,
        }
    }
}

/// A type definition for a field used to identify third party users or locations.
///
/// To create an instance of this type, first create a `FieldTypeInit` and convert it via
/// `FieldType::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FieldType {
    /// A regular expression for validation of a field's value.
    pub regexp: String,

    /// A placeholder serving as a valid example of the field value.
    pub placeholder: String,
}

/// Initial set of fields of `FieldType`.
///
/// This struct will not be updated even if additional fields are added to `FieldType` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct FieldTypeInit {
    /// A regular expression for validation of a field's value.
    pub regexp: String,

    /// A placeholder serving as a valid example of the field value.
    pub placeholder: String,
}

impl From<FieldTypeInit> for FieldType {
    fn from(init: FieldTypeInit) -> Self {
        let FieldTypeInit { regexp, placeholder } = init;
        Self { regexp, placeholder }
    }
}

/// A third party network location.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Location {
    /// An alias for a matrix room.
    pub alias: RoomAliasId,

    /// The protocol ID that the third party location is a part of.
    pub protocol: String,

    /// Information used to identify this third party location.
    pub fields: BTreeMap<String, String>,
}

impl Location {
    /// Creates a new `Location` with the given alias, protocol and fields.
    pub fn new(alias: RoomAliasId, protocol: String, fields: BTreeMap<String, String>) -> Self {
        Self { alias, protocol, fields }
    }
}

/// A third party network user.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct User {
    /// A matrix user ID representing a third party user.
    pub userid: UserId,

    /// The protocol ID that the third party user is a part of.
    pub protocol: String,

    /// Information used to identify this third party user.
    pub fields: BTreeMap<String, String>,
}

impl User {
    /// Creates a new `User` with the given userid, protocol and fields.
    pub fn new(userid: UserId, protocol: String, fields: BTreeMap<String, String>) -> Self {
        Self { userid, protocol, fields }
    }
}

/// The medium of a third party identifier.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
pub enum Medium {
    /// Email address identifier
    Email,

    /// Phone number identifier
    Msisdn,

    #[doc(hidden)]
    _Custom(String),
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
    pub validated_at: MilliSecondsSinceUnixEpoch,

    /// The time when the homeserver associated the third party identifier with the user.
    pub added_at: MilliSecondsSinceUnixEpoch,
}

/// Initial set of fields of `ThirdPartyIdentifier`.
///
/// This struct will not be updated even if additional fields are added to `ThirdPartyIdentifier`
/// in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct ThirdPartyIdentifierInit {
    /// The third party identifier address.
    pub address: String,

    /// The medium of third party identifier.
    pub medium: Medium,

    /// The time when the identifier was validated by the identity server.
    pub validated_at: MilliSecondsSinceUnixEpoch,

    /// The time when the homeserver associated the third party identifier with the user.
    pub added_at: MilliSecondsSinceUnixEpoch,
}

impl From<ThirdPartyIdentifierInit> for ThirdPartyIdentifier {
    fn from(init: ThirdPartyIdentifierInit) -> Self {
        let ThirdPartyIdentifierInit { address, medium, validated_at, added_at } = init;
        ThirdPartyIdentifier { address, medium, validated_at, added_at }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{Medium, ThirdPartyIdentifier};
    use crate::MilliSecondsSinceUnixEpoch;

    #[test]
    fn third_party_identifier_serde() {
        let third_party_id = ThirdPartyIdentifier {
            address: "monkey@banana.island".into(),
            medium: Medium::Email,
            validated_at: MilliSecondsSinceUnixEpoch(1_535_176_800_000_u64.try_into().unwrap()),
            added_at: MilliSecondsSinceUnixEpoch(1_535_336_848_756_u64.try_into().unwrap()),
        };

        let third_party_id_serialized = json!({
            "medium": "email",
            "address": "monkey@banana.island",
            "validated_at": 1_535_176_800_000_u64,
            "added_at": 1_535_336_848_756_u64
        });

        assert_eq!(to_json_value(third_party_id.clone()).unwrap(), third_party_id_serialized);
        assert_eq!(third_party_id, from_json_value(third_party_id_serialized).unwrap());
    }
}
