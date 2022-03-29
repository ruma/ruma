//! `GET /_matrix/app/*/thirdparty/user`
//!
//! Endpoint to retrieve an array of third party users from a Matrix User ID.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#get_matrixappv1thirdpartyuser

    use ruma_common::{api::ruma_api, thirdparty::User, UserId};

    ruma_api! {
        metadata: {
            description: "Retrieve an array of third party users from a Matrix User ID.",
            method: GET,
            name: "get_user_for_user_id",
            stable_path: "/_matrix/app/v1/thirdparty/user",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
            added: 1.0,
        }

        request: {
            /// The Matrix User ID to look up.
            #[ruma_api(query)]
            pub userid: &'a UserId,
        }

        response: {
            /// List of matched third party users.
            #[ruma_api(body)]
            pub users: Vec<User>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user id.
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
