//! Common types for user profile endpoints.

use std::borrow::Cow;

use ruma_macros::StringEnum;
#[cfg(feature = "unstable-msc4426")]
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value as JsonValue, from_value as from_json_value, to_value as to_json_value};

#[cfg(feature = "unstable-msc4426")]
use crate::SecondsSinceUnixEpoch;
use crate::{OwnedMxcUri, PrivOwnedStr};

mod profile_field_value_serde;
mod static_profile_field;
mod user_profile;

#[doc(hidden)]
pub use self::profile_field_value_serde::ProfileFieldValueVisitor;
pub use self::{static_profile_field::*, user_profile::*};

/// The possible fields of a user's [profile].
///
/// [profile]: https://spec.matrix.org/v1.18/client-server-api/#profiles
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

    /// The user's current status.
    ///
    /// This uses the unstable prefix defined in [MSC4426](https://github.com/matrix-org/matrix-spec-proposals/pull/4426).
    #[cfg(feature = "unstable-msc4426")]
    #[ruma_enum(rename = "org.matrix.msc4426.status")]
    Status,

    /// The user's call indicator.
    ///
    /// This uses the unstable prefix defined in [MSC4426](https://github.com/matrix-org/matrix-spec-proposals/pull/4426).
    #[cfg(feature = "unstable-msc4426")]
    #[ruma_enum(rename = "org.matrix.msc4426.call")]
    Call,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The possible values of a field of a user's [profile].
///
/// [profile]: https://spec.matrix.org/v1.18/client-server-api/#profiles
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

    /// The user's current status.
    ///
    /// This uses the unstable prefix defined in [MSC4426](https://github.com/matrix-org/matrix-spec-proposals/pull/4426).
    #[cfg(feature = "unstable-msc4426")]
    #[serde(rename = "org.matrix.msc4426.status")]
    Status(StatusProfileField),

    /// The user's call indicator.
    ///
    /// This uses the unstable prefix defined in [MSC4426](https://github.com/matrix-org/matrix-spec-proposals/pull/4426).
    #[cfg(feature = "unstable-msc4426")]
    #[serde(rename = "org.matrix.msc4426.call")]
    Call(CallProfileField),

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
            #[cfg(feature = "unstable-msc4426")]
            Self::Status(_) => ProfileFieldName::Status,
            #[cfg(feature = "unstable-msc4426")]
            Self::Call(_) => ProfileFieldName::Call,
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
            #[cfg(feature = "unstable-msc4426")]
            Self::Status(value) => {
                Cow::Owned(to_json_value(value).expect("value should serialize successfully"))
            }
            #[cfg(feature = "unstable-msc4426")]
            Self::Call(value) => {
                Cow::Owned(to_json_value(value).expect("value should serialize successfully"))
            }
            Self::_Custom(c) => Cow::Borrowed(&c.value),
        }
    }
}

/// A text-only field describing the user’s current state, along with an emoji.
///
/// The emoji can be useful as a compact summary, or just for fun.
#[cfg(feature = "unstable-msc4426")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StatusProfileField {
    /// The user’s chosen status text.
    ///
    /// Limited to 256 bytes. Does not support HTML.
    pub text: String,

    /// The user’s chosen status emoji.
    ///
    /// Limited to 32 bytes.
    pub emoji: String,
}

/// An indicator that the user is currently in a call, and optionally how long they’ve been in the
/// call.
#[cfg(feature = "unstable-msc4426")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CallProfileField {
    /// The time that the user joined the call.
    ///
    /// This allows users to see how long someone has been in a call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_joined_ts: Option<SecondsSinceUnixEpoch>,
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

#[cfg(test)]
mod tests {
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, owned_mxc_uri};
    use serde_json::{from_value as from_json_value, json};

    use super::ProfileFieldValue;
    #[cfg(feature = "unstable-msc4426")]
    use super::{CallProfileField, StatusProfileField};
    #[cfg(feature = "unstable-msc4426")]
    use crate::SecondsSinceUnixEpoch;

    #[test]
    fn serialize_profile_field_value() {
        // Avatar URL.
        let value = ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef"));
        assert_to_canonical_json_eq!(value, json!({ "avatar_url": "mxc://localhost/abcdef" }));

        // Display name.
        let value = ProfileFieldValue::DisplayName("Alice".to_owned());
        assert_to_canonical_json_eq!(value, json!({ "displayname": "Alice" }));

        // Custom field.
        let value = ProfileFieldValue::new("custom_field", "value".into()).unwrap();
        assert_to_canonical_json_eq!(value, json!({ "custom_field": "value" }));
    }

    #[test]
    fn deserialize_profile_field_value() {
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

    #[test]
    #[cfg(feature = "unstable-msc4426")]
    fn serialize_profile_status() {
        // Status.
        let value = ProfileFieldValue::Status(StatusProfileField {
            text: "Away".to_owned(),
            emoji: "🌴".to_owned(),
        });
        assert_to_canonical_json_eq!(
            value,
            json!({ "org.matrix.msc4426.status": { "text": "Away", "emoji": "🌴" } })
        );

        // Call.
        let value = ProfileFieldValue::Call(CallProfileField {
            call_joined_ts: Some(SecondsSinceUnixEpoch(1_770_140_640.try_into().unwrap())),
        });
        assert_to_canonical_json_eq!(
            value,
            json!({ "org.matrix.msc4426.call": { "call_joined_ts": 1_770_140_640 } })
        );
    }

    #[test]
    #[cfg(feature = "unstable-msc4426")]
    fn deserialize_profile_status() {
        // Status.
        let json =
            json!({ "org.matrix.msc4426.status": { "text": "Be right back", "emoji": "☕️" } });
        assert_eq!(
            from_json_value::<ProfileFieldValue>(json).unwrap(),
            ProfileFieldValue::Status(StatusProfileField {
                text: "Be right back".to_owned(),
                emoji: "☕️".to_owned(),
            })
        );

        // Call.
        let json = json!({ "org.matrix.msc4426.call": { "call_joined_ts": 1_168_380_060 } });
        assert_eq!(
            from_json_value::<ProfileFieldValue>(json).unwrap(),
            ProfileFieldValue::Call(CallProfileField {
                call_joined_ts: Some(SecondsSinceUnixEpoch(1_168_380_060.try_into().unwrap())),
            })
        );
    }
}
