//! `GET /_matrix/federation/*/query/profile`
//!
//! Get profile information, such as a display name or avatar, for a given user.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1queryprofile

    use ruma_common::{
        MxcUri, UserId,
        api::{request, response},
        metadata,
        serde::StringEnum,
    };

    use crate::{PrivOwnedStr, authentication::ServerSignatures};

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/query/profile",
    }

    /// Request type for the `get_profile_information` endpoint.
    #[request]
    pub struct Request {
        /// User ID to query.
        #[ruma_api(query)]
        pub user_id: UserId,

        /// Profile field to query.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub field: Option<ProfileField>,
    }

    /// Response type for the `get_profile_information` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// Display name of the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// Avatar URL for the user's avatar.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub avatar_url: Option<MxcUri>,

        /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
        ///
        /// This uses the unstable prefix in
        /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
        #[cfg(feature = "unstable-msc2448")]
        #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
        pub blurhash: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: UserId) -> Self {
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
    ///
    /// This type can hold an arbitrary string. To build this with a custom value, convert it from a
    /// string with `::from()` / `.into()`. To check for values that are not available as a
    /// documented variant here, use its string representation, obtained through
    /// [`.as_str()`](Self::as_str()).
    #[derive(Clone, StringEnum)]
    #[non_exhaustive]
    pub enum ProfileField {
        /// Display name of the user.
        #[ruma_enum(rename = "displayname")]
        DisplayName,

        /// Avatar URL for the user's avatar.
        #[ruma_enum(rename = "avatar_url")]
        AvatarUrl,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}
