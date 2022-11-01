//! `GET /_matrix/app/*/thirdparty/user/{protocol}`
//!
//! Endpoint to retrieve a Matrix User ID linked to a user on the third party network, given a set
//! of user parameters.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/application-service-api/#get_matrixappv1thirdpartyuserprotocol

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::User,
    };

    const METADATA: Metadata = metadata! {
        description: "Fetches third party users for a protocol.",
        method: GET,
        name: "get_user_for_protocol",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/thirdparty/user/:protocol",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The protocol used to communicate to the third party network.
        #[ruma_api(path)]
        pub protocol: &'a str,

        /// One or more custom fields that are passed to the AS to help identify the user.
        // The specification is incorrect for this parameter. See [matrix-spec#560](https://github.com/matrix-org/matrix-spec/issues/560).
        #[ruma_api(query_map)]
        pub fields: BTreeMap<String, String>,
    }

    #[response]
    pub struct Response {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
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
}
