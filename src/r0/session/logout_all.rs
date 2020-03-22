//! [POST /_matrix/client/r0/logout/all](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-logout-all)

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Invalidates all access tokens for a user, so that they can no longer be used for authorization.",
        method: POST,
        name: "logout_all",
        path: "/_matrix/client/r0/logout/all",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {}

    error: crate::Error
}
