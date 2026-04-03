//! `PUT /_matrix/client/*/admin/lock/{userId}`
//!
//! Sets the locked status of a particular server-local user.
//!
//! The user calling this endpoint MUST be a server admin. The client SHOULD check that the user is
//! allowed to lock other users at the `GET /capabilities` endpoint prior to using this endpoint.
//!
//! In order to prevent user enumeration, servers MUST ensure that authorization is checked prior to
//! trying to do account lookups.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#put_matrixclientv1adminlockuserid

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
            unstable("uk.timedout.msc4323") => "/_matrix/client/unstable/uk.timedout.msc4323/admin/lock/{user_id}",
            1.18 => "/_matrix/client/v1/admin/lock/{user_id}",
        }
    }

    /// Request type for the `lock_user` endpoint.
    #[request]
    pub struct Request {
        /// The user to change the locked status of.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// Whether to lock the target account.
        pub locked: bool,
    }

    /// Response type for the `lock_user` endpoint.
    #[response]
    pub struct Response {
        /// Whether the target account is locked.
        pub locked: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and locked status.
        pub fn new(user_id: OwnedUserId, locked: bool) -> Self {
            Self { user_id, locked }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given locked status.
        pub fn new(locked: bool) -> Self {
            Self { locked }
        }
    }
}
