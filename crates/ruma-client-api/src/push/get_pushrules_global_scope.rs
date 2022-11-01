//! `GET /_matrix/client/*/pushrules/global/`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3pushrules

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        push::Ruleset,
    };

    const METADATA: Metadata = metadata! {
        description: "Retrieve all push rulesets in the global scope for this user.",
        method: GET,
        name: "get_pushrules_global_scope",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/global/",
            1.1 => "/_matrix/client/v3/pushrules/global/",
        }
    };

    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    #[response(error = crate::Error)]
    pub struct Response {
        /// The global ruleset.
        #[ruma_api(body)]
        pub global: Ruleset,
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
