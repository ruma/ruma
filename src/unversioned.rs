//! Endpoints that cannot change with new versions of the Matrix specification.

/// [GET /_matrix/client/versions](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-versions)
pub mod get_supported_versions {
    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Get the versions of the client-server API supported by this homeserver.",
            method: Method::Get,
            name: "api_versions",
            path: "/_matrix/client/versions",
            rate_limited: false,
            requires_authentication: true,
        }

        request {}

        response {
            /// A list of Matrix client API protocol versions supported by the homeserver.
            pub versions: Vec<String>,
        }
    }
}
