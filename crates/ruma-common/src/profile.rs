//! Common types for user profile endpoints.

use std::borrow::Cow;
use std::collections::HashMap;
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, from_value as from_json_value, to_value as to_json_value};

use crate::{OwnedMxcUri, PrivOwnedStr};

mod profile_field_value_serde;

#[doc(hidden)]
pub use self::profile_field_value_serde::ProfileFieldValueVisitor;

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

/// Represents a user's whole profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile(HashMap<String, JsonValue>);

impl Profile {
    /// Creates a new, empty profile.
    pub fn new() -> Self {
        Profile(HashMap::new())
    }

    /// Inserts a field and its value into the profile.
    pub fn insert(&mut self, field: ProfileFieldValue) {
        self.0.insert(field.field_name().as_str().to_owned(), field.value().into_owned());
    }

    /// Gets a single field, returning its value.
    ///
    /// If there is no field with the given key, None is returned.
    /// If the field exists, but there is an error deserializing it, Some(Err(...)) is returned.
    pub fn get(&self, field: &str) -> Option<serde_json::Result<ProfileFieldValue>> {
        self.0.get(field).map(|value| ProfileFieldValue::new(field, value.to_owned()))
    }

    /// Removes a single field from the profile.
    pub fn remove(&mut self, field: &str) {
        self.0.remove(field);
    }

    /// Clears the profile of all keys.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns the number of keys in the profile.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the profile contains no keys.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the fields of the profile in no defined order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &JsonValue)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, owned_mxc_uri};
    use serde_json::{from_value as from_json_value, json};

    use super::{Profile, ProfileFieldValue};

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
    fn deserialize_profile() {
        let json = json!({ "avatar_url": "mxc://localhost/abcdef", "display_name": "Alice", "io.ruma.custom_field": {"key": "value"} });
        let profile = from_json_value::<Profile>(json).unwrap();

        assert_eq!(profile.len(), 3);
        let avatar_url = profile.get("avatar_url").expect("avatar_url should not be None").unwrap();
        assert_eq!(avatar_url.field_name().as_str(), "avatar_url");
        assert_eq!(avatar_url.value().as_str(), Some("mxc://localhost/abcdef"));
        let displayname = profile.get("display_name").expect("display_name should not be None").unwrap();
        assert_eq!(displayname.field_name().as_str(), "display_name");
        assert_eq!(displayname.value().as_str(), Some("Alice"));
        let custom_field = profile.get("io.ruma.custom_field").expect("io.ruma.custom_field should not be None").unwrap();
        assert_eq!(custom_field.field_name().as_str(), "io.ruma.custom_field");
        assert_to_canonical_json_eq!(custom_field.value().into_owned(), json!({"key": "value"}));
    }

    #[test]
    fn serialize_custom_profile() {
        let mut profile = Profile::new();
        profile.insert(ProfileFieldValue::DisplayName("Alice".to_owned()));
        profile.insert(ProfileFieldValue::TimeZone("Etc/UTC".to_owned()));
        profile.insert(ProfileFieldValue::AvatarUrl(owned_mxc_uri!("mxc://localhost/abcdef")));
        profile.insert(ProfileFieldValue::new("io.ruma.custom_field", json!({"key": "value"})).unwrap());

        assert_to_canonical_json_eq!(profile, json!({
            "displayname": "Alice",
            "m.tz": "Etc/UTC",
            "avatar_url": "mxc://localhost/abcdef",
            "io.ruma.custom_field": {"key": "value"}
        }));
    }
}
