//! `POST /_matrix/client/*/users/{userId}/report`
//!
//! Report a user as inappropriate.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3usersuseridreport

    use ruma_common::{
        UserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4260/users/{user_id}/report",
            1.14 => "/_matrix/client/v3/users/{user_id}/report",
        }
    }

    /// Request type for the `report_user` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the user to report.
        #[ruma_api(path)]
        pub user_id: UserId,

        /// The reason to report the user, may be empty.
        pub reason: String,
    }

    /// Response type for the `report_user` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and reason.
        pub fn new(user_id: UserId, reason: String) -> Self {
            Self { user_id, reason }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
