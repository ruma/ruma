//! [POST /_matrix/client/r0/user_directory/search](https://matrix.org/docs/spec/client_server/r0.6.1#post-matrix-client-r0-user-directory-search)

use js_int::{uint, UInt};
use ruma_api::ruma_api;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Performs a search for users.",
        method: POST,
        name: "search_users",
        path: "/_matrix/client/r0/user_directory/search",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The term to search for.
        pub search_term: &'a str,

        /// The maximum number of results to return.
        ///
        /// Defaults to 10.
        #[serde(default = "default_limit", skip_serializing_if = "is_default_limit")]
        pub limit: UInt,

        /// Language tag to determine the collation to use for the (case-insensitive) search.
        ///
        /// See [MDN] for the syntax.
        ///
        /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language#Syntax
        #[ruma_api(header = ACCEPT_LANGUAGE)]
        pub language: Option<String>,
    }

    response: {
        /// Ordered by rank and then whether or not profile info is available.
        pub results: Vec<User>,

        /// Indicates if the result list has been truncated by the limit.
        pub limited: bool,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given search term.
    pub fn new(search_term: &'a str) -> Self {
        Self { search_term, limit: default_limit(), language: None }
    }
}

impl Response {
    /// Creates a new `Response` with the given results and limited flag
    pub fn new(results: Vec<User>, limited: bool) -> Self {
        Self { results, limited }
    }
}

fn default_limit() -> UInt {
    uint!(10)
}

fn is_default_limit(limit: &UInt) -> bool {
    limit == &default_limit()
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
