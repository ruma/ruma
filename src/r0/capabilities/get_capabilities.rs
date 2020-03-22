//! [GET /_matrix/client/r0/capabilities](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-capabilities)

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

ruma_api! {
    metadata {
        description: "Gets information about the server's supported feature set and other relevant capabilities.",
        method: GET,
        name: "get_capabilities",
        path: "/_matrix/client/r0/capabilities",
        rate_limited: true,
        requires_authentication: true
    }

    request {}

    response {
        /// The capabilities the server supports
        pub capabilities: Capabilities,
    }

    error: crate::Error
}

/// Contains information about all the capabilities that the server supports.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub custom_capabilities: HashMap<String, Value>,
}

/// Information about the m.change_password capability
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ChangePasswordCapability {
    /// True if the user can change their password, false otherwise.
    pub enabled: bool,
}

/// Information about the m.room_versions capability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomVersionsCapability {
    /// The default room version the server is using for new rooms.
    pub default: String,

    /// A detailed description of the room versions the server supports.
    pub available: HashMap<String, RoomVersionStability>,
}

/// The stability of a room version
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RoomVersionStability {
    /// An unstable room version
    #[serde(rename = "stable")]
    Stable,

    /// A stable room version
    #[serde(rename = "unstable")]
    Unstable,
}
