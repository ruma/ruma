//! [GET /_matrix/app/v1/thirdparty/user/{protocol}](https://matrix.org/docs/spec/application_service/r0.1.2#get-matrix-app-v1-thirdparty-user-protocol)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::thirdparty::User;

ruma_api! {
    metadata: {
        description: "Fetches third party users for a protocol.",
        method: GET,
        name: "get_user_for_protocol",
        path: "/_matrix/app/v1/thirdparty/user/:protocol",
        rate_limited: false,
        authentication: QueryOnlyAccessToken,
    }

    request: {
        /// The protocol used to communicate to the third party network.
        #[ruma_api(path)]
        pub protocol: &'a str,

        /// One or more custom fields that are passed to the AS to help identify the user.
        // The specification is incorrect for this parameter. See matrix-org/matrix-doc#2352.
        #[ruma_api(query_map)]
        pub fields: BTreeMap<String, String>,
    }

    response: {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given protocol name.
    pub fn new(protocol: &'a str) -> Self {
        Self { protocol, fields: BTreeMap::new() }
    }
}

impl Response {
    /// Creates a new `Response` with the given users.
    pub fn new(users: Vec<User>) -> Self {
        Self { users }
    }
}
