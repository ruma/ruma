//! Endpoints for Voice over IP.

/// [GET /_matrix/client/r0/voip/turnServer](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-voip-turnserver)
pub mod get_turn_server_info {
    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub username: String,
        pub password: String,
        pub uris: Vec<String>,
        pub ttl: u64
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "_matrix/client/r0/voip/turnServer"
        }

        fn name() -> &'static str {
            "turn_server_info"
        }

        fn description() -> &'static str {
            "Get credentials for the client to use when initiating VoIP calls."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }
}
