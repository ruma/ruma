//! `GET /_matrix/app/*/users/{userId}`
//!
//! Endpoint to query the existence of a given user ID.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/application-service-api/#get_matrixappv1usersuserid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, UserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/users/:user_id",
        }
    };

    /// Request type for the `query_user_id` endpoint.
    #[request]
    pub struct Request<'a> {
        /// The user ID being queried.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    /// Response type for the `query_user_id` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: &'a UserId) -> Self {
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
