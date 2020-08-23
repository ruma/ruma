//! [GET /_matrix/federation/v1/query/profile](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-query-profile)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get profile information, such as a display name or avatar, for a given user.",
        name: "get_profile_information",
        method: GET,
        path: "/_matrix/federation/v1/query/profile",
        rate_limited: false,
        requires_authentication: true,
    }

    #[non_exhaustive]
    request: {
        /// User ID to query.
        #[ruma_api(query)]
        pub user_id: &'a UserId,

        /// Profile field to query.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub field: Option<ProfileField>,
    }

    #[derive(Default)]
    #[non_exhaustive]
    response: {
        /// Display name of the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// Avatar URL for the user's avatar.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user id.
    pub fn new(user_id: &'a UserId) -> Self {
        Self { user_id, field: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// Profile fields to specify in query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProfileField {
    /// Display name of the user.
    #[serde(rename = "displayname")]
    DisplayName,

    /// Avatar URL for the user's avatar.
    #[serde(rename = "avatar_url")]
    AvatarUrl,
}
