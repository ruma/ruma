//! [GET /_matrix/federation/v1/openid/userinfo](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-openid-userinfo)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Exchanges an OpenID access token for information about the user who generated the token.",
        method: GET,
        name: "get_openid_userinfo",
        path: "/_matrix/federation/v1/openid/userinfo",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The OpenID access token to get information about the owner for.
        #[ruma_api(query)]
        pub access_token: &'a str,
    }

    response: {
        /// The Matrix User ID who generated the token.
        pub sub: UserId,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given access token.
    pub fn new(access_token: &'a str) -> Self {
        Self { access_token }
    }
}

impl Response {
    /// Creates a new `Response` with the given user id.
    pub fn new(sub: UserId) -> Self {
        Self { sub }
    }
}
