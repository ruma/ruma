//! `GET /_matrix/client/*/profile/{userId}`
//!
//! Get all profile information of an user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3profileuserid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/profile/:user_id",
            1.1 => "/_matrix/client/v3/profile/:user_id",
        }
    };

    /// Request type for the `get_profile` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose profile will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `get_profile` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The user's avatar URL, if set.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub avatar_url: Option<OwnedMxcUri>,

        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
        #[cfg(feature = "unstable-msc2448")]
        #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
        pub blurhash: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given avatar URL and display name.
        pub fn new(avatar_url: Option<OwnedMxcUri>, displayname: Option<String>) -> Self {
            Self {
                avatar_url,
                displayname,
                #[cfg(feature = "unstable-msc2448")]
                blurhash: None,
            }
        }
    }
}
