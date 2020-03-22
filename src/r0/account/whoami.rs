//! [GET /_matrix/client/r0/account/whoami](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-account-whoami)

use ruma_api::ruma_api;

ruma_api! {
    metadata {
        description: "Get information about the owner of a given access token.",
        method: GET,
        name: "whoami",
        path: "/_matrix/client/r0/account/whoami",
        rate_limited: true,
        requires_authentication: true,
    }

    request {}

    response {
        /// The id of the user that owns the access token.
        pub user_id: String,
    }

    error: crate::Error
}
