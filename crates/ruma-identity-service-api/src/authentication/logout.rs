//! `POST /_matrix/identity/*/account/logout`
//!
//! Logs out the access token, preventing it from being used to authenticate future requests to the
//! server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#post_matrixidentityv2accountlogout

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Logs out the access token, preventing it from being used to authenticate future requests to the server.",
        method: POST,
        name: "logout",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/account/logout",
        }
    };

    #[request]
    #[derive(Default)]
    pub struct Request {}

    #[response]
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
