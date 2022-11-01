//! `GET /_matrix/client/*/thirdparty/user`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3thirdpartyuser

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::User,
        UserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Retrieve an array of third party users from a Matrix User ID.",
        method: GET,
        name: "get_user_for_user_id",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/thirdparty/user",
            1.1 => "/_matrix/client/v3/thirdparty/user",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The Matrix User ID to look up.
        #[ruma_api(query)]
        pub userid: &'a UserId,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID.
        pub fn new(userid: &'a UserId) -> Self {
            Self { userid }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given users.
        pub fn new(users: Vec<User>) -> Self {
            Self { users }
        }
    }
}
