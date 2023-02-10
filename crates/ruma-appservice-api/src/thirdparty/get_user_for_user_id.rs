//! `GET /_matrix/app/*/thirdparty/user`
//!
//! Retrieve an array of third party users from a Matrix User ID.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#get_matrixappv1thirdpartyuser

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::User,
        OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/thirdparty/user",
        }
    };

    /// Request type for the `get_user_for_user_id` endpoint.
    #[request]
    pub struct Request {
        /// The Matrix User ID to look up.
        #[ruma_api(query)]
        pub userid: OwnedUserId,
    }

    /// Response type for the `get_user_for_user_id` endpoint.
    #[response]
    pub struct Response {
        /// List of matched third party users.
        #[ruma_api(body)]
        pub users: Vec<User>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(userid: OwnedUserId) -> Self {
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
