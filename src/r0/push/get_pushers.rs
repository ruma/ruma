//! [GET /_matrix/client/r0/pushers](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushers)

use ruma_api::ruma_api;

use super::Pusher;

ruma_api! {
    metadata {
        description: "Gets all currently active pushers for the authenticated user.",
        method: GET,
        name: "get_pushers",
        path: "/_matrix/client/r0/pushers",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        /// An array containing the current pushers for the user.
        pub pushers: Vec<Pusher>
    }

    error: crate::Error
}
