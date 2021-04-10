//! [PUT /_matrix/client/r0/profile/{userId}/avatar_url](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-profile-userid-avatar-url)

use ruma_api::ruma_api;
use ruma_identifiers::{MxcUri, UserId};

ruma_api! {
    metadata: {
        description: "Set the avatar URL of the user.",
        method: PUT,
        name: "set_avatar_url",
        path: "/_matrix/client/r0/profile/:user_id/avatar_url",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The user whose avatar URL will be set.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The new avatar URL for the user.
        ///
        /// `None` is used to unset the avatar.
        ///
        /// If you activate the `compat` feature, this field being an empty string in JSON will give
        /// you `None` here.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
            serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
        )]
        pub avatar_url: Option<&'a MxcUri>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and avatar URL.
    pub fn new(user_id: &'a UserId, avatar_url: Option<&'a MxcUri>) -> Self {
        Self { user_id, avatar_url }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use matches::assert_matches;
    use ruma_api::IncomingRequest as _;

    use super::IncomingRequest;

    #[test]
    fn deserialize_unset_request() -> Result<(), Box<dyn std::error::Error>> {
        assert_matches!(
            IncomingRequest::try_from_http_request(
                http::Request::builder()
                    .method("PUT")
                    .uri("https://bar.org/_matrix/client/r0/profile/@foo:bar.org/avatar_url")
                    .body(&[] as &[u8])?,
            )?,
            IncomingRequest { user_id, avatar_url: None } if user_id == "@foo:bar.org"
        );

        #[cfg(feature = "compat")]
        assert_matches!(
            IncomingRequest::try_from_http_request(
                http::Request::builder()
                    .method("PUT")
                    .uri("https://bar.org/_matrix/client/r0/profile/@foo:bar.org/avatar_url")
                    .body(std::io::Cursor::new(
                        serde_json::to_vec(&serde_json::json!({ "avatar_url": "" }))?,
                    ))?,
            )?,
            IncomingRequest { user_id, avatar_url: None } if user_id == "@foo:bar.org"
        );

        Ok(())
    }
}
