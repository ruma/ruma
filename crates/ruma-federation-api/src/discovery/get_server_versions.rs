//! Endpoint to receive metadata about implemented matrix versions.
//!
//! Get the supported matrix versions of this homeserver

pub mod msc3723 {
    //! [GET /_matrix/federation/versions](https://github.com/matrix-org/matrix-spec-proposals/pull/3723)

    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "/_matrix/federation/unstable/org.matrix.msc3723/versions",
    }

    /// Request type for the `get_server_versions` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_server_versions` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// A list of Matrix Server API protocol versions supported by the homeserver.
        pub versions: Vec<String>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
