//! `GET /_matrix/client/*/auth_issuer`
//!
//! Get the OpenID Connect Provider that is trusted by the homeserver.
//!
//! This endpoint has been replaced by [`get_authorization_server_metadata`] in [MSC2965].
//!
//! [`get_authorization_server_metadata`]: super::get_authorization_server_metadata
//! [MSC2965]: https://github.com/matrix-org/matrix-spec-proposals/pull/2965

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
    #[deprecated = "Replaced by the get_authorization_server_metadata endpoint."]
    pub struct Request {}

    /// Request type for the `auth_issuer` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The OpenID Connect Provider that is trusted by the homeserver.
        pub issuer: String,
    }

    #[allow(deprecated)]
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
