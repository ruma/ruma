//! `GET /_matrix/client/*/auth_issuer`
//!
//! Get the OpenID Connect Provider that is trusted by the homeserver.

pub mod msc2965 {
    //! `MSC2965` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2965

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc2965/auth_issuer",
        }
    };

    /// Request type for the `auth_issuer` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Request type for the `auth_issuer` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The OpenID Connect Provider that is trusted by the homeserver.
        pub issuer: String,
    }

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given issuer.
        pub fn new(issuer: String) -> Self {
            Self { issuer }
        }
    }
}
