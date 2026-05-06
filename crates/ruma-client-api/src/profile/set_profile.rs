//! `PUT /_matrix/client/*/profile/{userId}`
//!
//! Set the entire profile for a user

pub mod unstable {
    //! `/unstable/com.beeper.msc4437/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4437

    use ruma_common::{
        OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };
    use std::collections::BTreeMap;
    use ruma_common::profile::{ProfileFieldName, ProfileFieldValue};

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("com.beeper.msc4437") => "/_matrix/client/unstable/com.beeper.msc4437/profile/{user_id}",
        }
    }

    /// Request type for the `set_profile` endpoint.
    #[request]
    pub struct Request {
        /// The user whose profile will be set.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The new profile for the user.
        #[ruma_api(body)]
        pub data: BTreeMap<ProfileFieldName, ProfileFieldValue>,
    }

    /// Response type for the `set_profile` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and profile data.
        pub fn new(user_id: OwnedUserId, data: BTreeMap<ProfileFieldName, ProfileFieldValue>) -> Self {
            Self { user_id, data }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}

