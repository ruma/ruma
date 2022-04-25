//! `GET /_matrix/federation/*/query/profile`
//!
//! Endpoint to query profile information with a user id and optional field.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1queryprofile

    use ruma_common::{api::ruma_api, serde::StringEnum, OwnedMxcUri, UserId};

    use crate::PrivOwnedStr;

    ruma_api! {
        metadata: {
            description: "Get profile information, such as a display name or avatar, for a given user.",
            name: "get_profile_information",
            method: GET,
            stable_path: "/_matrix/federation/v1/query/profile",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        request: {
            /// User ID to query.
            #[ruma_api(query)]
            pub user_id: &'a UserId,

            /// Profile field to query.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[ruma_api(query)]
            pub field: Option<&'a ProfileField>,
        }

        #[derive(Default)]
        response: {
            /// Display name of the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<String>,

            /// Avatar URL for the user's avatar.
            ///
            /// If you activate the `compat` feature, this field being an empty string in JSON will result
            /// in `None` here during deserialization.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[cfg_attr(
                feature = "compat",
                serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
            )]
            pub avatar_url: Option<OwnedMxcUri>,

            /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
            ///
            /// This uses the unstable prefix in
            /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
            #[cfg(feature = "unstable-msc2448")]
            #[serde(
                rename = "xyz.amorgan.blurhash",
                alias = "blurhash",
                skip_serializing_if = "Option::is_none"
            )]
            pub blurhash: Option<String>,
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
    ///
    /// This type can hold an arbitrary string. To build this with a custom value, convert it from a
    /// string with `::from() / .into()`. To check for formats that are not available as a
    /// documented variant here, use its string representation, obtained through `.as_str()`.
    #[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
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

    impl ProfileField {
        /// Creates a string slice from this `ProfileField`.
        pub fn as_str(&self) -> &str {
            self.as_ref()
        }
    }
}
