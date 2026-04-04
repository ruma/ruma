//! `GET /_matrix/client/*/admin/suspend/{userId}`
//!
//! Gets information about the suspended status of a particular server-local user.
//!
//! The user calling this endpoint MUST be a server admin.
//!
//! In order to prevent user enumeration, servers MUST ensure that authorization is checked prior to
//! trying to do account lookups.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#get_matrixclientv1adminsuspenduserid

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable("uk.timedout.msc4323") => "/_matrix/client/unstable/uk.timedout.msc4323/admin/suspend/{user_id}",
            1.18 => "/_matrix/client/v1/admin/suspend/{user_id}",
        }
    }

    /// Request type for the `is_user_suspended` endpoint.
    #[request]
    pub struct Request {
        /// The user to look up.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `is_user_suspended` endpoint.
    #[response]
    pub struct Response {
        /// Whether the target account is suspended.
        pub suspended: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given suspended status.
        pub fn new(suspended: bool) -> Self {
            Self { suspended }
        }
    }
}
