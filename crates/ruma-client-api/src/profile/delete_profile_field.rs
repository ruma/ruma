//! `DELETE /_matrix/client/*/profile/{userId}/{key_name}`
//!
//! Delete a field on the profile of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/4133

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    use crate::profile::ProfileFieldName;

    const METADATA: Metadata = metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("uk.tcpip.msc4133") => "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
            // 1.15 => "/_matrix/client/v3/profile/{user_id}/{field}",
        }
    };

    /// Request type for the `delete_profile_field` endpoint.
    #[request(error = crate::Error)]
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
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
