//! [POST /_matrix/client/r0/user_directory/search](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-user-directory-search)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Performs a search for users on the homeserver.",
        method: POST,
        name: "search_users",
        path: "/_matrix/client/r0/user_directory/search",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The term to search for.
        pub search_term: String,
        /// The maximum number of results to return.
        ///
        /// Defaults to 10.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    response {
        /// Ordered by rank and then whether or not profile info is available.
        pub results: Vec<User>,
        /// Indicates if the result list has been truncated by the limit.
        pub limited: bool,
    }

    error: crate::Error
}

/// User data as result of a search.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    /// The user's matrix user ID.
    pub user_id: UserId,
    /// The display name of the user, if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The avatar url, as an MXC, if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}
