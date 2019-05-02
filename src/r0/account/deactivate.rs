//! [POST /_matrix/client/r0/account/deactivate](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-account-deactivate)
// TODO: missing request parameters

use ruma_api_macros::ruma_api;

ruma_api! {
    metadata {
        description: "Deactivate the current user's account.",
        method: POST,
        name: "deactivate",
        path: "/_matrix/client/r0/account/deactivate",
        rate_limited: true,
        requires_authentication: true,
    }

    request {}

    response {}
}
