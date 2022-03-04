//! `GET /_matrix/app/*/users/{userId}`
//!
//! Endpoint to query the existence of a given user ID.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#get_matrixappv1usersuserid

    use ruma_common::{api::ruma_api, UserId};

    ruma_api! {
        metadata: {
            description: "This endpoint is invoked by the homeserver on an application service to query the existence of a given user ID.",
            method: GET,
            name: "query_user_id",
            stable_path: "/_matrix/app/v1/users/:user_id",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
            added: 1.0,
        }

        request: {
            /// The user ID being queried.
            #[ruma_api(path)]
            pub user_id: &'a UserId,
        }

        #[derive(Default)]
        response: {}
    }

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
