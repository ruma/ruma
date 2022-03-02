//! `GET /_matrix/client/*/pushrules/global/`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3pushrules

    use ruma_common::{api::ruma_api, push::Ruleset};

    ruma_api! {
        metadata: {
            description: "Retrieve all push rulesets in the global scope for this user.",
            method: GET,
            name: "get_pushrules_global_scope",
            r0_path: "/_matrix/client/r0/pushrules/global/",
            stable_path: "/_matrix/client/v3/pushrules/global/",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// The global ruleset.
            #[ruma_api(body)]
            pub global: Ruleset,
        }

        error: crate::Error
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given global ruleset.
        pub fn new(global: Ruleset) -> Self {
            Self { global }
        }
    }
}
