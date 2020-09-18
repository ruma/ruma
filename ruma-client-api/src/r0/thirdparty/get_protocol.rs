//! [GET /_matrix/client/r0/thirdparty/protocol/{protocol}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-thirdparty-protocol-protocol)

use ruma_api::ruma_api;
use ruma_common::thirdparty::Protocol;

ruma_api! {
    metadata: {
        description: "Fetches the metadata from the homeserver about a particular third party protocol.",
        method: GET,
        name: "get_protocol",
        path: "/_matrix/client/r0/thirdparty/protocol/:protocol",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The name of the protocol.
        #[ruma_api(path)]
        pub protocol: &'a str,
    }

    response: {
        /// Metadata about the protocol.
        #[ruma_api(body)]
        pub protocol: Protocol,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given protocol name.
    pub fn new(protocol: &'a str) -> Self {
        Self { protocol }
    }
}

impl Response {
    /// Creates a new `Response` with the given procotol.
    pub fn new(protocol: Protocol) -> Self {
        Self { protocol }
    }
}
