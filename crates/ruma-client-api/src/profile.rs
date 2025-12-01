//! Endpoints for user profiles.

use std::borrow::Cow;

#[cfg(feature = "client")]
use ruma_common::api::{
    MatrixVersion,
    path_builder::{StablePathSelector, VersionHistory},
};
use ruma_common::{OwnedMxcUri, serde::StringEnum};
use serde::Serialize;
use serde_json::{Value as JsonValue, from_value as from_json_value, to_value as to_json_value};

pub mod delete_profile_field;
pub mod get_avatar_url;
pub mod get_display_name;
pub mod get_profile;
pub mod get_profile_field;
mod profile_field_serde;
pub mod set_avatar_url;
pub mod set_display_name;
pub mod set_profile_field;
mod static_profile_field;

pub use self::static_profile_field::*;

/// The possible fields of a user's [profile].
///
/// [profile]: https://spec.matrix.org/latest/client-server-api/#profiles
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProfileFieldName {
    /// The user's avatar URL.
    AvatarUrl,

    /// The user's display name.
    #[ruma_enum(rename = "displayname")]
    DisplayName,

    /// The user's time zone.
    #[ruma_enum(rename = "m.tz")]
    TimeZone,

    #[doc(hidden)]
    _Custom(crate::PrivOwnedStr),
}

impl ProfileFieldName {
    /// Whether this field name existed already before custom fields were officially supported in
    /// profiles.
    #[cfg(feature = "client")]
    fn existed_before_extended_profiles(&self) -> bool {
        matches!(self, Self::AvatarUrl | Self::DisplayName)
    }
}

/// The possible values of a field of a user's [profile].
///
/// [profile]: https://spec.matrix.org/latest/client-server-api/#profiles
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProfileFieldValue {
    /// The user's avatar URL.
    AvatarUrl(OwnedMxcUri),

    /// The user's display name.
    #[serde(rename = "displayname")]
    DisplayName(String),

    /// The user's time zone.
    #[serde(rename = "m.tz")]
    TimeZone(String),

    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomProfileFieldValue),
}

impl ProfileFieldValue {
    /// Construct a new `ProfileFieldValue` with the given field and value.
    ///
    /// Prefer to use the public variants of `ProfileFieldValue` where possible; this constructor is
    /// meant to be used for unsupported fields only and does not allow setting arbitrary data for
    /// supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `field` is known and serialization of `value` to the corresponding
    /// `ProfileFieldValue` variant fails.
    pub fn new(field: &str, value: JsonValue) -> serde_json::Result<Self> {
        Ok(match field {
            "avatar_url" => Self::AvatarUrl(from_json_value(value)?),
            "displayname" => Self::DisplayName(from_json_value(value)?),
            "m.tz" => Self::TimeZone(from_json_value(value)?),
            _ => Self::_Custom(CustomProfileFieldValue { field: field.to_owned(), value }),
        })
    }

    /// The name of the field for this value.
    pub fn field_name(&self) -> ProfileFieldName {
        match self {
            Self::AvatarUrl(_) => ProfileFieldName::AvatarUrl,
            Self::DisplayName(_) => ProfileFieldName::DisplayName,
            Self::TimeZone(_) => ProfileFieldName::TimeZone,
            Self::_Custom(CustomProfileFieldValue { field, .. }) => field.as_str().into(),
        }
    }

    /// Returns the value of the field.
    ///
    /// Prefer to use the public variants of `ProfileFieldValue` where possible; this method is
    /// meant to be used for custom fields only.
    pub fn value(&self) -> Cow<'_, JsonValue> {
        match self {
            Self::AvatarUrl(value) => {
                Cow::Owned(to_json_value(value).expect("value should serialize successfully"))
            }
            Self::DisplayName(value) => {
                Cow::Owned(to_json_value(value).expect("value should serialize successfully"))
            }
            Self::TimeZone(value) => {
                Cow::Owned(to_json_value(value).expect("value should serialize successfully"))
            }
            Self::_Custom(c) => Cow::Borrowed(&c.value),
        }
    }
}

/// A custom value for a user's profile field.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct CustomProfileFieldValue {
    /// The name of the field.
    field: String,

    /// The value of the field
    value: JsonValue,
}

/// Endpoint version history valid only for profile fields that didn't exist before Matrix 1.16.
#[cfg(feature = "client")]
const EXTENDED_PROFILE_FIELD_HISTORY: VersionHistory = VersionHistory::new(
    &[(
        Some("uk.tcpip.msc4133"),
        "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
    )],
    &[(
        StablePathSelector::Version(MatrixVersion::V1_16),
        "/_matrix/client/v3/profile/{user_id}/{field}",
    )],
    None,
    None,
);

#[cfg(test)]
mod tests {
    use ruma_common::owned_mxc_uri;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::ProfileFieldValue;

    #[test]
    fn serialize_profile_field_value() {
        // Avatar URL.
        let value = ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef"));
        assert_eq!(
            to_json_value(value).unwrap(),
            json!({ "avatar_url": "mxc://localhost/abcdef" })
        );

        // Display name.
        let value = ProfileFieldValue::DisplayName("Alice".to_owned());
        assert_eq!(to_json_value(value).unwrap(), json!({ "displayname": "Alice" }));

        // Custom field.
        let value = ProfileFieldValue::new("custom_field", "value".into()).unwrap();
        assert_eq!(to_json_value(value).unwrap(), json!({ "custom_field": "value" }));
    }

    #[test]
    fn deserialize_any_profile_field_value() {
        // Avatar URL.
        let json = json!({ "avatar_url": "mxc://localhost/abcdef" });
        assert_eq!(
            from_json_value::<ProfileFieldValue>(json).unwrap(),
            ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef"))
        );

        // Display name.
        let json = json!({ "displayname": "Alice" });
        assert_eq!(
            from_json_value::<ProfileFieldValue>(json).unwrap(),
            ProfileFieldValue::DisplayName("Alice".to_owned())
        );

        // Custom field.
        let json = json!({ "custom_field": "value" });
        let value = from_json_value::<ProfileFieldValue>(json).unwrap();
        assert_eq!(value.field_name().as_str(), "custom_field");
        assert_eq!(value.value().as_str(), Some("value"));

        // Error if the object is empty.
        let json = json!({});
        from_json_value::<ProfileFieldValue>(json).unwrap_err();
    }
}
