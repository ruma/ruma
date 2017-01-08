//! Endpoints for server administration.

/// [GET /_matrix/client/r0/admin/whois/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-admin-whois-userid)
pub mod get_user_info {
    use ruma_identifiers::UserId;

    use std::collections::HashMap;

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

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug)]
    pub struct PathParams {
        /// The user to look up.
        pub user_id: UserId,
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        /// The Matrix user ID of the user.
        pub user_id: UserId,
        /// A map of the user's device identifiers to information about that device.
        pub devices: HashMap<String, DeviceInfo>,
    }

    /// Information about a user session.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SessionInfo {
        /// A list of connections in this session.
        pub connections: Vec<ConnectionInfo>,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/admin/whois/{}",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/admin/whois/:user_id"
        }

        fn name() -> &'static str {
            "get_user_info"
        }

        fn description() -> &'static str {
            "Get information about a particular user."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
