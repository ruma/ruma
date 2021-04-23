//! [GET /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-profile-userid-avatar-url)

use ruma_api::ruma_api;
use ruma_identifiers::{MxcUri, UserId};

ruma_api! {
    metadata: {
        description: "Get the avatar URL of a user.",
        method: GET,
        name: "get_avatar_url",
        path: "/_matrix/client/r0/profile/:user_id/avatar_url",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The user whose avatar URL will be retrieved.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[derive(Default)]
    response: {
        /// The user's avatar URL, if set.
        ///
        /// If you activate the `compat` feature, this field being an empty string in JSON will give
        /// you `None` here.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
            serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
        )]
        pub avatar_url: Option<MxcUri>
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID.
    pub fn new(user_id: &'a UserId) -> Self {
        Self { user_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given avatar URL.
    pub fn new(avatar_url: Option<MxcUri>) -> Self {
        Self { avatar_url }
    }
}
