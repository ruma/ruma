//! [GET /_matrix/client/r0/capabilities](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-capabilities)

use ruma_api::ruma_api;
use ruma_identifiers::RoomVersionId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

ruma_api! {
    metadata: {
        description: "Gets information about the server's supported feature set and other relevant capabilities.",
        method: GET,
        name: "get_capabilities",
        path: "/_matrix/client/r0/capabilities",
        rate_limited: true,
        authentication: AccessToken
    }

    #[derive(Default)]
    request: {}

    response: {
        /// The capabilities the server supports
        pub capabilities: Capabilities,
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
    /// Creates a new `Response` with the given capabilities.
    pub fn new(capabilities: Capabilities) -> Self {
        Self { capabilities }
    }
}

impl From<Capabilities> for Response {
    fn from(capabilities: Capabilities) -> Self {
        Self::new(capabilities)
    }
}

/// Contains information about all the capabilities that the server supports.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Capabilities {
    /// Capability to indicate if the user can change their password.
    #[serde(rename = "m.change_password", skip_serializing_if = "Option::is_none")]
    pub change_password: Option<ChangePasswordCapability>,

    /// The room versions the server supports.
    #[serde(rename = "m.room_versions", skip_serializing_if = "Option::is_none")]
    pub room_versions: Option<RoomVersionsCapability>,

    /// Any other custom capabilities that the server supports outside of the specification,
    /// labeled using the Java package naming convention and stored as arbitrary JSON values.
    #[serde(flatten)]
    pub custom_capabilities: BTreeMap<String, JsonValue>,
}

impl Capabilities {
    /// Creates empty `Capabilities`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Information about the m.change_password capability
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ChangePasswordCapability {
    /// True if the user can change their password, false otherwise.
    pub enabled: bool,
}

impl ChangePasswordCapability {
    /// Creates a new `ChangePasswordCapability` with the given enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

/// Information about the m.room_versions capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomVersionsCapability {
    /// The default room version the server is using for new rooms.
    pub default: RoomVersionId,

    /// A detailed description of the room versions the server supports.
    pub available: BTreeMap<RoomVersionId, RoomVersionStability>,
}

impl RoomVersionsCapability {
    /// Creates a new `RoomVersionsCapability` with the given default room version ID and room
    /// version descriptions.
    pub fn new(
        default: RoomVersionId,
        available: BTreeMap<RoomVersionId, RoomVersionStability>,
    ) -> Self {
        Self { default, available }
    }
}

/// The stability of a room version
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RoomVersionStability {
    /// Support for the given version is stable.
    #[serde(rename = "stable")]
    Stable,

    /// Support for the given version is unstable.
    #[serde(rename = "unstable")]
    Unstable,
}
