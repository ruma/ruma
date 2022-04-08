//! `GET /_matrix/client/*/admin/whois/{userId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3adminwhoisuserid

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, MilliSecondsSinceUnixEpoch, OwnedUserId, UserId};
    use serde::{Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Get information about a particular user.",
            method: GET,
            name: "get_user_info",
            r0_path: "/_matrix/client/r0/admin/whois/:user_id",
            stable_path: "/_matrix/client/v3/admin/whois/:user_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The user to look up.
            #[ruma_api(path)]
            pub user_id: &'a UserId,
        }

        #[derive(Default)]
        response: {
            /// The Matrix user ID of the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub user_id: Option<OwnedUserId>,

            /// A map of the user's device identifiers to information about that device.
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            pub devices: BTreeMap<String, DeviceInfo>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: &'a UserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// Information about a user's device.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct DeviceInfo {
        /// A list of user sessions on this device.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub sessions: Vec<SessionInfo>,
    }

    impl DeviceInfo {
        /// Create a new `DeviceInfo` with no sessions.
        pub fn new() -> Self {
            Self::default()
        }
    }

    /// Information about a user session.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct SessionInfo {
        /// A list of connections in this session.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub connections: Vec<ConnectionInfo>,
    }

    impl SessionInfo {
        /// Create a new `SessionInfo` with no connections.
        pub fn new() -> Self {
            Self::default()
        }
    }

    /// Information about a connection in a user session.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct ConnectionInfo {
        /// Most recently seen IP address of the session.
        pub ip: Option<String>,

        /// Time when that the session was last active.
        pub last_seen: Option<MilliSecondsSinceUnixEpoch>,

        /// User agent string last seen in the session.
        pub user_agent: Option<String>,
    }

    impl ConnectionInfo {
        /// Create a new `ConnectionInfo` with all fields set to `None`.
        pub fn new() -> Self {
            Self::default()
        }
    }
}
