//! `DELETE /_matrix/client/*/profile/{userId}/{key_name}`
//!
//! Delete a field on the profile of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#delete_matrixclientv3profileuseridkeyname

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        profile::ProfileFieldName,
    };

    metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("uk.tcpip.msc4133") => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
            1.16 => "/_matrix/client/v3/profile/{user_id}/{field}",
        }
    }

    /// Request type for the `delete_profile_field` endpoint.
    #[request]
    pub struct Request {
        /// The user whose profile will be updated.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The profile field to delete.
        #[ruma_api(path)]
        pub field: ProfileFieldName,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and field.
        pub fn new(user_id: OwnedUserId, field: ProfileFieldName) -> Self {
            Self { user_id, field }
        }
    }

    /// Response type for the `delete_profile_field` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
