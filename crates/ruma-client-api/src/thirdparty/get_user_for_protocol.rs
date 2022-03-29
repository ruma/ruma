//! `GET /_matrix/client/*/thirdparty/user/{protocol}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3thirdpartyuserprotocol

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, thirdparty::User};

    ruma_api! {
        metadata: {
            description: "Fetches third party users for a protocol.",
            method: GET,
            name: "get_user_for_protocol",
            r0_path: "/_matrix/client/r0/thirdparty/user/:protocol",
            stable_path: "/_matrix/client/v3/thirdparty/user/:protocol",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The protocol used to communicate to the third party network.
            #[ruma_api(path)]
            pub protocol: &'a str,

            /// One or more custom fields that are passed to the AS to help identify the user.
            // The specification is incorrect for this parameter. See [matrix-spec#560](https://github.com/matrix-org/matrix-spec/issues/560).
            #[ruma_api(query_map)]
            pub fields: BTreeMap<String, String>,
        }

        response: {
            /// List of matched third party users.
            #[ruma_api(body)]
            pub users: Vec<User>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given protocol.
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
}
