//! Endpoint to receive metadata about implemented matrix versions.

pub mod msc3723 {
    //! [GET /_matrix/federation/versions](https://github.com/matrix-org/matrix-doc/pull/3723)

    use ruma_api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Get the supported matrix versions of this homeserver",
            method: GET,
            name: "get_server_versions",
            path: "/_matrix/federation/unstable/org.matrix.msc3723/versions",
            unstable: "/_matrix/federation/unstable/org.matrix.msc3723/versions",
            rate_limited: false,
            authentication: None,
        }

        #[derive(Default)]
        request: {}

        #[derive(Default)]
        response: {
            /// A list of Matrix Server API protocol versions supported by the homeserver.
            pub versions: Vec<String>,
        }
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
