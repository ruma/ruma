//! [GET /_matrix/client/r0/account/3pid](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-account-3pid)

use ruma_api::ruma_api;
use ruma_common::thirdparty::ThirdPartyIdentifier;

ruma_api! {
    metadata: {
        description: "Get a list of 3rd party contacts associated with the user's account.",
        method: GET,
        name: "get_3pids",
        r0_path: "/_matrix/client/r0/account/3pid",
        stable_path: "/_matrix/client/v3/account/3pid",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.0,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of third party identifiers the homeserver has associated with the user's account.
        #[serde(default)]
        #[cfg_attr(not(feature = "compat"), serde(skip_serializing_if = "Vec::is_empty"))]
        pub threepids: Vec<ThirdPartyIdentifier>,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given third party identifiers.
    pub fn new(threepids: Vec<ThirdPartyIdentifier>) -> Self {
        Self { threepids }
    }
}
