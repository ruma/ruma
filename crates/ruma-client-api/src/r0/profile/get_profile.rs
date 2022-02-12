//! [GET /_matrix/client/r0/profile/{userId}](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-profile-userid)

use ruma_api::ruma_api;
use ruma_identifiers::{MxcUri, UserId};

ruma_api! {
    metadata: {
        description: "Get all profile information of an user.",
        method: GET,
        name: "get_profile",
        r0: "/_matrix/client/r0/profile/:user_id",
        stable: "/_matrix/client/v3/profile/:user_id",
        rate_limited: false,
        authentication: None,
        added: 1.0,
    }

    request: {
        /// The user whose profile will be retrieved.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[derive(Default)]
    response: {
        /// The user's avatar URL, if set.
        ///
        /// If you activate the `compat` feature, this field being an empty string in JSON will result
        /// in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
            serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
        )]
        pub avatar_url: Option<Box<MxcUri>>,

        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
        #[cfg(feature = "unstable-msc2448")]
        #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
        pub blurhash: Option<String>,
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
    /// Creates a new `Response` with the given avatar URL and display name.
    pub fn new(avatar_url: Option<Box<MxcUri>>, displayname: Option<String>) -> Self {
        Self {
            avatar_url,
            displayname,
            #[cfg(feature = "unstable-msc2448")]
            blurhash: None,
        }
    }
}
