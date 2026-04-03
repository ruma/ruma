//! `GET /_matrix/client/*/pushrules/`
//!
//! Retrieve all push rulesets for this user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#get_matrixclientv3pushrules

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        push::Ruleset,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushrules/",
            1.1 => "/_matrix/client/v3/pushrules/",
        }
    }

    /// Request type for the `get_pushrules_all` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_pushrules_all` endpoint.
    #[response]
    pub struct Response {
        /// The global ruleset.
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
