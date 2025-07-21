//! `GET /_matrix/app/*/thirdparty/user/{protocol}`
//!
//! Retrieve a Matrix User ID linked to a user on the third party network, given a set of user
//! parameters.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#get_matrixappv1thirdpartyuserprotocol

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::User,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/thirdparty/user/{protocol}",
        }
    };

    /// Request type for the `get_user_for_protocol` endpoint.
    #[request]
    pub struct Request {
        /// The protocol used to communicate to the third party network.
        #[ruma_api(path)]
        pub protocol: String,

        /// One or more custom fields that are passed to the AS to help identify the user.
        #[ruma_api(query_all)]
        pub fields: BTreeMap<String, String>,
    }

    /// Response type for the `get_user_for_protocol` endpoint.
    #[response]
    pub struct Response {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
    }

    impl Request {
        /// Creates a new `Request` with the given protocol name.
        pub fn new(protocol: String) -> Self {
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
