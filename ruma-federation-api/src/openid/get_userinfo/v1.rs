//! [GET /_matrix/federation/v1/openid/userinfo](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-openid-userinfo)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

ruma_api! {
    metadata: {
        description: "Exchanges an OpenID access token for information about the user who generated the token. Currently this only exposes the Matrix User ID of the owner.",
        method: GET,
        name: "openid_userinfo",
        path: "/_matrix/federation/v1/openid/userinfo",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        /// The OpenID access token to get information about the owner for.
        #[ruma_api(query)]
        pub access_token: String,
    }

    response: {
        /// Information about the user who generated the OpenID access token.
        pub sub: UserId,
    }
}
