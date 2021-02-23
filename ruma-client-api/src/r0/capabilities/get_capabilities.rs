//! [GET /_matrix/client/r0/capabilities](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-capabilities)

use std::collections::BTreeMap;

use maplit::btreemap;
use ruma_api::ruma_api;
use ruma_identifiers::RoomVersionId;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use super::Capabilities;

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

/// Information about the m.change_password capability
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ChangePasswordCapability {
    /// `true` if the user can change their password, `false` otherwise.
    pub enabled: bool,
}

impl ChangePasswordCapability {
    /// Creates a new `ChangePasswordCapability` with the given enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.enabled
    }
}

impl Default for ChangePasswordCapability {
    fn default() -> Self {
        Self { enabled: true }
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

    /// Returns whether all fields have their default value.
    pub fn is_default(&self) -> bool {
        self.default == RoomVersionId::Version1
            && self.available.len() == 1
            && self
                .available
                .get(&RoomVersionId::Version1)
                .map(|stability| *stability == RoomVersionStability::Stable)
                .unwrap_or(false)
    }
}

impl Default for RoomVersionsCapability {
    fn default() -> Self {
        Self {
            default: RoomVersionId::Version1,
            available: btreemap! { RoomVersionId::Version1 => RoomVersionStability::Stable },
        }
    }
}

/// The stability of a room version
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
pub enum RoomVersionStability {
    /// Support for the given version is stable.
    Stable,

    /// Support for the given version is unstable.
    Unstable,

    #[doc(hidden)]
    _Custom(String),
}
