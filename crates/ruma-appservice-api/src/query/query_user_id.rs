//! `GET /_matrix/app/*/users/{userId}`
//!
//! Endpoint to query the existence of a given user ID.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#get_matrixappv1usersuserid

    use ruma_common::{
        UserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        path: "/_matrix/app/v1/users/{user_id}",
    }

    /// Request type for the `query_user_id` endpoint.
    #[request]
    pub struct Request {
        /// The user ID being queried.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    /// Response type for the `query_user_id` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: UserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
