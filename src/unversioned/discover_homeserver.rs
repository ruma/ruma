use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Get discovery information about the domain.",
        method: GET,
        name: "discover_homeserver",
        path: "/.well-known/matrix/server",
        rate_limited: false,
        requires_authentication: false,
    }

    request {}

    response {
        /// The server name to delegate server-server communciations to, with optional port.
        #[serde(rename = "m.homeserver")]
        pub homeserver: String,
    }
}
