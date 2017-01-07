//! Endpoints for user presence.

/// [PUT /_matrix/client/r0/presence/{userId}/status](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-presence-userid-status)
pub mod set_presence {
    use ruma_identifiers::UserId;
    use ruma_events::presence::PresenceState;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,
        pub presence: PresenceState
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Put
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/presence/{}/status",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/presence/:user_id/status"
        }

        fn name() -> &'static str {
            "set_presence"
        }

        fn description() -> &'static str {
            "Set presence status for this user."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [GET /_matrix/client/r0/presence/{userId}/status](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-presence-userid-status)
pub mod get_presence {
    use ruma_identifiers::UserId;
    use ruma_events::presence::PresenceState;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub currently_active: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_active_ago: Option<u64>,
        pub presence: PresenceState
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
                "/_matrix/client/r0/presence/{}/status",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/presence/:user_id/status"
        }

        fn name() -> &'static str {
            "get_presence"
        }

        fn description() -> &'static str {
            "Get presence status for this user."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}

/// [POST /_matrix/client/r0/presence/list/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-presence-list-userid)
pub mod update_presence_subscriptions {
    use ruma_identifiers::UserId;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    /// This API endpoint's body parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        drop: Vec<UserId>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        invite: Vec<UserId>
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/presence/list/{}",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/presence/list/:user_id"
        }

        fn name() -> &'static str {
            "update_presence_subscriptions"
        }

        fn description() -> &'static str {
            "Update the presence subscriptions of the user."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}

/// [GET /_matrix/client/r0/presence/list/{userId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-presence-list-userid)
pub mod get_subscribed_presences {
    use ruma_identifiers::UserId;
    use ruma_events::presence::PresenceEvent;

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: UserId
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Vec<PresenceEvent>;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/presence/list/{}",
                params.user_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/presence/list/:user_id"
        }

        fn name() -> &'static str {
            "get_subscribed_presences"
        }

        fn description() -> &'static str {
            "Get the precence status from the user's subscriptions."
        }

        fn requires_authentication() -> bool {
            // TODO: not sure why this does not require authentication
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
