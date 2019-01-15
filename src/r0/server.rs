//! Endpoints for server administration.

/// [GET /_matrix/client/r0/admin/whois/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-admin-whois-userid)
pub mod get_user_info {
    use std::collections::HashMap;

    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;
    use serde_derive::{Deserialize, Serialize};

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
    }

    /// Information about a connection in a user session.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ConnectionInfo {
        /// Most recently seen IP address of the session.
        pub ip: String,
        /// Unix timestamp that the session was last active.
        pub last_seen: u64,
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
}
