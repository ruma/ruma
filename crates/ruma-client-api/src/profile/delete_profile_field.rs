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

    #[cfg(feature = "unstable-msc4466")]
    use crate::profile::PropagateTo;

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

        /// The propagation mode to use for this profile update.
        #[cfg(feature = "unstable-msc4466")]
        #[ruma_api(query)]
        #[serde(rename = "computer.gingershaped.msc4466.propagate_to")]
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub propagate_to: PropagateTo,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and field.
        pub fn new(user_id: OwnedUserId, field: ProfileFieldName) -> Self {
            Self {
                user_id,
                field,
                #[cfg(feature = "unstable-msc4466")]
                propagate_to: PropagateTo::default(),
            }
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
