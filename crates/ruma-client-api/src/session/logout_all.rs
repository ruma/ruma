//! `POST /_matrix/client/*/logout/all`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3logoutall

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Invalidates all access tokens for a user, so that they can no longer be used for authorization.",
        method: POST,
        name: "logout_all",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/logout/all",
            1.1 => "/_matrix/client/v3/logout/all",
        }
    };

    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
