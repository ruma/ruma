//! Endpoints for third party lookups

pub mod get_location_for_protocol;
pub mod get_location_for_room_alias;
pub mod get_protocol;
pub mod get_protocols;
pub mod get_user_for_protocol;
pub mod get_user_for_user_id;

use std::collections::HashMap;

use ruma_identifiers::{RoomAliasId, UserId};

use serde::{Deserialize, Serialize};

/// Metadata about a third party protocol.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Protocol {
    /// Fields which may be used to identify a third party user.
    pub user_fields: Vec<String>,
    /// Fields which may be used to identify a third party location.
    pub location_fields: Vec<String>,
    /// A content URI representing an icon for the third party protocol.
    pub icon: String,
    /// The type definitions for the fields defined in `user_fields` and `location_fields`.
    pub field_types: HashMap<String, FieldType>,
    /// A list of objects representing independent instances of configuration.
    pub instances: Vec<ProtocolInstance>,
}

/// Metadata about an instance of a third party protocol.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolInstance {
    /// A human-readable description for the protocol, such as the name.
    pub desc: String,
    /// An optional content URI representing the protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Preset values for `fields` the client may use to search by.
    pub fields: HashMap<String, String>,
    /// A unique identifier across all instances.
    pub network_id: String,
}

/// A type definition for a field used to identify third party users or locations.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FieldType {
    /// A regular expression for validation of a field's value.
    pub regexp: String,
    /// A placeholder serving as a valid example of the field value.
    pub placeholder: String,
}

/// A third party network location.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Location {
    /// An alias for a matrix room.
    pub alias: RoomAliasId,
    /// The protocol ID that the third party location is a part of.
    pub protocol: String,
    /// Information used to identify this third party location.
    pub fields: HashMap<String, String>,
}

/// A third party network user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    /// A matrix user ID representing a third party user.
    pub userid: UserId,
    /// The protocol ID that the third party user is a part of.
    pub protocol: String,
    /// Information used to identify this third party user.
    pub fields: HashMap<String, String>,
}
