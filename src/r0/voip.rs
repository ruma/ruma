//! Endpoints for Voice over IP.

/// GET /_matrix/client/r0/voip/turnServer
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-voip-turnserver)
pub mod get_turn_server_info {
    /// Details about this API endpoint.
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
            "/_matrix/client/r0/voip/turnServer".to_string()
        }

        fn router_path() -> String {
            "_matrix/client/r0/voip/turnServer".to_string()
        }
    }
}
