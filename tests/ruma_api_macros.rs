#![feature(associated_consts, proc_macro, try_from)]

extern crate futures;
extern crate hyper;
extern crate ruma_api;
extern crate ruma_api_macros;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate url;

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
