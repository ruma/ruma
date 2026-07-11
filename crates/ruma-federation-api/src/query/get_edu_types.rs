//! GET `/_matrix/federation/*/query/edutypes`
//!
//! Determine what types of EDUs a server wishes to receive.

pub mod unstable {
    //! `/unstable/io.fsky.vel/edutypes` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4373

    use ruma_common::{
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "/_matrix/federation/unstable/io.fsky.vel/edutypes"
    }

    /// Request type for the `edutypes` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `edutypes` endpoint.
    #[response]
    pub struct Response {
        /// Whether presence EDUs should be sent/received
        #[serde(rename = "m.presence", default = "ruma_common::serde::default_true")]
        pub presence: bool,

        /// Whether read receipt EDUs should be sent/received
        #[serde(rename = "m.receipt", default = "ruma_common::serde::default_true")]
        pub receipt: bool,

        /// Whether typing EDUs should be sent/received
        #[serde(rename = "m.typing", default = "ruma_common::serde::default_true")]
        pub typing: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given event id.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with all EDU flags set to `true`.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for Response {
        fn default() -> Self {
            Self { presence: true, receipt: true, typing: true }
        }
    }
}
