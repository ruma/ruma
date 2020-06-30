//! [POST /_matrix/client/r0/logout](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-logout)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Log out of the homeserver.",
        method: POST,
        name: "logout",
        path: "/_matrix/client/r0/logout",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {}

    response: {}

    error: crate::Error
}
