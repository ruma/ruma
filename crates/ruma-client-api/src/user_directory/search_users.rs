//! `POST /_matrix/client/*/user_directory/search`
//!
//! Performs a search for users.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3user_directorysearch

    use http::header::ACCEPT_LANGUAGE;
    use js_int::{uint, UInt};
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user_directory/search",
            1.1 => "/_matrix/client/v3/user_directory/search",
        }
    };

    /// Request type for the `search_users` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The term to search for.
        pub search_term: String,

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

    /// Response type for the `search_users` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Ordered by rank and then whether or not profile info is available.
        pub results: Vec<User>,

        /// Indicates if the result list has been truncated by the limit.
        pub limited: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given search term.
        pub fn new(search_term: String) -> Self {
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
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct User {
        /// The user's matrix user ID.
        pub user_id: OwnedUserId,

        /// The display name of the user, if one exists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<String>,

        /// The avatar url, as an MXC, if one exists.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub avatar_url: Option<OwnedMxcUri>,
    }

    impl User {
        /// Create a new `User` with the given `UserId`.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, display_name: None, avatar_url: None }
        }
    }
}
