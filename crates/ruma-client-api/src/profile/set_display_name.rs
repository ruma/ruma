//! `PUT /_matrix/client/*/profile/{userId}/displayname`
//!
//! Set the display name of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.15/client-server-api/#put_matrixclientv3profileuseriddisplayname

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    #[cfg(feature = "unstable-msc4466")]
    use crate::profile::PropagateTo;

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/profile/{user_id}/displayname",
            1.1 => "/_matrix/client/v3/profile/{user_id}/displayname",
        }
    }

    /// Request type for the `set_display_name` endpoint.
    #[request]
    pub struct Request {
        /// The user whose display name will be set.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The new display name for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// The propagation mode to use for this profile update.
        #[cfg(feature = "unstable-msc4466")]
        #[ruma_api(query)]
        #[serde(rename = "computer.gingershaped.msc4466.propagate_to")]
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub propagate_to: PropagateTo,
    }

    /// Response type for the `set_display_name` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and display name.
        #[deprecated = "Use the set_profile_field endpoint instead."]
        pub fn new(user_id: OwnedUserId, displayname: Option<String>) -> Self {
            Self {
                user_id,
                displayname,
                #[cfg(feature = "unstable-msc4466")]
                propagate_to: PropagateTo::default(),
            }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
