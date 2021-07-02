//! [GET /_matrix/federation/v1/query/profile](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-query-profile)

use ruma_api::ruma_api;
use ruma_identifiers::{MxcUri, UserId};
use ruma_serde::StringEnum;

ruma_api! {
    metadata: {
        description: "Get profile information, such as a display name or avatar, for a given user.",
        name: "get_profile_information",
        method: GET,
        path: "/_matrix/federation/v1/query/profile",
        rate_limited: false,
        authentication: ServerSignatures,
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
        /// If you activate the `compat` feature, this field being an empty string in JSON will give
        /// you `None` here.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
            serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
        )]
        pub avatar_url: Option<MxcUri>,
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
/// This type can hold an arbitrary string. To check for formats that are not available as a
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
    _Custom(String),
}

impl ProfileField {
    /// Creates a string slice from this `ProfileField`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
