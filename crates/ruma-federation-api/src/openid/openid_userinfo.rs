//! `GET /_matrix/openid/*/userinfo`
//!
//! Exchange an OpenID access token for information about the user who generated the token.

pub mod v1 {
    //! `/v1/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4447/

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "/_matrix/openid/v1/userinfo",
    }

    /// Request type for the `openid_userinfo` endpoint.
    #[request]
    pub struct Request {
        /// The OpenID access token to get information about the owner for.
        #[ruma_api(query)]
        pub access_token: String,
    }

    /// Response type for the `openid_userinfo` endpoint.
    #[response]
    pub struct Response {
        /// The Matrix User ID who generated the token.
        pub sub: OwnedUserId,
    }

    impl Request {
        /// Creates a new `Request` with the given access token.
        pub fn new(access_token: String) -> Self {
            Self { access_token }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user id.
        pub fn new(sub: OwnedUserId) -> Self {
            Self { sub }
        }
    }
}
