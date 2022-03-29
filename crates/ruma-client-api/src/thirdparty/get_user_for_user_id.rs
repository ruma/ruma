//! `GET /_matrix/client/*/thirdparty/user`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3thirdpartyuser

    use ruma_common::{api::ruma_api, thirdparty::User, UserId};

    ruma_api! {
        metadata: {
            description: "Retrieve an array of third party users from a Matrix User ID.",
            method: GET,
            name: "get_user_for_user_id",
            r0_path: "/_matrix/client/r0/thirdparty/user",
            stable_path: "/_matrix/client/v3/thirdparty/user",
            rate_limited: false,
            authentication: AccessToken,
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

        error: crate::Error
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
