#![feature(proc_macro)]

extern crate hyper;
extern crate ruma_api;
extern crate ruma_api_macros;

pub mod get_supported_versions {
    use ruma_api_macros::ruma_api;

    ruma_api! {
        const METADATA: Metadata = Metadata {
            description: "Get the versions of the client-server API supported by this homeserver.",
            method: Method::Get,
            name: "api_versions",
            path: "/_matrix/client/versions",
            rate_limited: false,
            requires_authentication: true,
        };

        struct Request;

        struct Response {
            /// A list of Matrix client API protocol versions supported by the homeserver.
            pub versions: Vec<String>,
        }
    }
}
