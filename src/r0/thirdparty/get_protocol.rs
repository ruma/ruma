//! [GET /_matrix/client/r0/thirdparty/protocol/{protocol}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-protocol-protocol)

use ruma_api::ruma_api;

use super::Protocol;

ruma_api! {
    metadata {
        description: "Fetches the metadata from the homeserver about a particular third party protocol.",
        method: GET,
        name: "get_protocol",
        path: "/_matrix/client/r0/thirdparty/protocol/:protocol",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The name of the protocol.
        #[ruma_api(path)]
        pub protocol: String,
    }

    response {
        /// Metadata about the protocol.
        #[ruma_api(body)]
        pub protocol: Protocol,
    }
}
