//! [GET /_matrix/app/v1/thirdparty/protocol/{protocol}](https://matrix.org/docs/spec/application_service/r0.1.2#get-matrix-app-v1-thirdparty-protocol-protocol)

use ruma_api::ruma_api;

use super::Protocol;

ruma_api! {
    metadata: {
        description: "Fetches the metadata from the homeserver about a particular third party protocol.",
        method: GET,
        name: "get_protocol",
        path: "/_matrix/app/v1/thirdparty/protocol/:protocol",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The name of the protocol.
        #[ruma_api(path)]
        pub protocol: String,
    }

    response: {
        /// Metadata about the protocol.
        #[ruma_api(body)]
        pub protocol: Protocol,
    }
}
