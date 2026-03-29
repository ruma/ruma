//! `PUT /_matrix/client/*/admin/suspend/{userId}`
//!
//! Sets the suspended status of a particular server-local user.
//!
//! The user calling this endpoint MUST be a server admin. The client SHOULD check that the user is
//! allowed to suspend other users at the `GET /capabilities` endpoint prior to using this endpoint.
//!
//! In order to prevent user enumeration, servers MUST ensure that authorization is checked prior to
//! trying to do account lookups.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#put_matrixclientv1adminsuspenduserid

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable("uk.timedout.msc4323") => "/_matrix/client/unstable/uk.timedout.msc4323/admin/suspend/{user_id}",
            1.18 => "/_matrix/client/v1/admin/suspend/{user_id}",
        }
    }

    /// Request type for the `suspend_user` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user to change the suspended status of.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// Whether to suspend the target account.
        pub suspended: bool,
    }

    /// Response type for the `suspend_user` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Whether the target account is suspended.
        pub suspended: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and suspended status.
        pub fn new(user_id: OwnedUserId, suspended: bool) -> Self {
            Self { user_id, suspended }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given suspended status.
        pub fn new(suspended: bool) -> Self {
            Self { suspended }
        }
    }
}
