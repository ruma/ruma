//! [GET /_matrix/client/r0/admin/whois/{userId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-admin-whois-userid)

use std::collections::HashMap;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Get information about a particular user.",
        method: GET,
        name: "get_user_info",
        path: "/_matrix/client/r0/admin/whois/:user_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The user to look up.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// The Matrix user ID of the user.
        pub user_id: UserId,
        /// A map of the user's device identifiers to information about that device.
        pub devices: HashMap<String, DeviceInfo>,
    }

    error: crate::Error
}

/// Information about a connection in a user session.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionInfo {
    /// Most recently seen IP address of the session.
    pub ip: String,
    /// Unix timestamp that the session was last active.
    pub last_seen: UInt,
    /// User agent string last seen in the session.
    pub user_agent: String,
}

/// Information about a user's device.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceInfo {
    /// A list of user sessions on this device.
    pub sessions: Vec<SessionInfo>,
}

/// Information about a user session.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionInfo {
    /// A list of connections in this session.
    pub connections: Vec<ConnectionInfo>,
}
