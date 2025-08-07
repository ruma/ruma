//! Endpoints for user profiles.

#[cfg(feature = "unstable-msc4133")]
use std::{borrow::Cow, fmt, marker::PhantomData};

#[cfg(feature = "unstable-msc4133")]
use ruma_common::{
    serde::{OrdAsRefStr, PartialOrdAsRefStr},
    OwnedMxcUri,
};
#[cfg(feature = "unstable-msc4133")]
use serde::{
    de::{self, DeserializeOwned, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};
#[cfg(feature = "unstable-msc4133")]
use serde_json::{from_value as from_json_value, to_value as to_json_value, Value as JsonValue};

#[cfg(feature = "unstable-msc4133")]
pub mod delete_profile_field;
pub mod get_avatar_url;
pub mod get_display_name;
pub mod get_profile;
#[cfg(feature = "unstable-msc4133")]
pub mod get_profile_field;
pub mod set_avatar_url;
pub mod set_display_name;
#[cfg(feature = "unstable-msc4133")]
pub mod set_profile_field;

/// Trait implemented by types representing a field in a user's profile having a statically-known
/// name.
#[cfg(feature = "unstable-msc4133")]
pub trait StaticProfileField {
    /// The type for the value of the field.
    type Value: Sized + Serialize + DeserializeOwned;

    /// The string representation of this field.
    const NAME: &str;
}

/// Helper type to deserialize any type that implements [`StaticProfileField`].
#[cfg(feature = "unstable-msc4133")]
struct StaticProfileFieldVisitor<F: StaticProfileField>(PhantomData<F>);

#[cfg(feature = "unstable-msc4133")]
impl<'de, F: StaticProfileField> Visitor<'de> for StaticProfileFieldVisitor<F> {
    type Value = Option<F::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a map with optional key `{}` and value", F::NAME)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut found = false;

        while let Some(key) = map.next_key::<Cow<'_, str>>()? {
            if key == F::NAME {
                found = true;
                break;
            }
        }

        if !found {
            return Ok(None);
        }

        Ok(Some(map.next_value()?))
    }
}

/// The user's avatar URL.
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "unstable-msc4133")]
#[allow(clippy::exhaustive_structs)]
pub struct AvatarUrl;

#[cfg(feature = "unstable-msc4133")]
impl StaticProfileField for AvatarUrl {
    type Value = OwnedMxcUri;
    const NAME: &str = "avatar_url";
}

/// The user's display name.
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "unstable-msc4133")]
#[allow(clippy::exhaustive_structs)]
pub struct DisplayName;

#[cfg(feature = "unstable-msc4133")]
impl StaticProfileField for DisplayName {
    type Value = String;
    const NAME: &str = "displayname";
}

/// The possible fields of a user's profile.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[cfg(feature = "unstable-msc4133")]
#[derive(Clone, PartialEq, Eq, PartialOrdAsRefStr, OrdAsRefStr, ruma_common::serde::StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProfileFieldName {
    /// The user's avatar URL.
    AvatarUrl,

    /// The user's display name.
    #[ruma_enum(rename = "displayname")]
    DisplayName,

    #[doc(hidden)]
    _Custom(crate::PrivOwnedStr),
}

/// The possible values of a field of a user's profile.
#[cfg(feature = "unstable-msc4133")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProfileFieldValue {
    /// The user's avatar URL.
    AvatarUrl(OwnedMxcUri),

    /// The user's display name.
    #[serde(rename = "displayname")]
    DisplayName(String),

    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomProfileFieldValue),
}

#[cfg(feature = "unstable-msc4133")]
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
            _ => Self::_Custom(CustomProfileFieldValue { field: field.to_owned(), value }),
        })
    }

    /// The name of the field for this value.
    pub fn field_name(&self) -> ProfileFieldName {
        match self {
            Self::AvatarUrl(_) => ProfileFieldName::AvatarUrl,
            Self::DisplayName(_) => ProfileFieldName::DisplayName,
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
            Self::_Custom(c) => Cow::Borrowed(&c.value),
        }
    }
}

/// A custom value for a user's profile field.
#[cfg(feature = "unstable-msc4133")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct CustomProfileFieldValue {
    /// The name of the field.
    field: String,

    /// The value of the field
    value: serde_json::Value,
}

#[cfg(feature = "unstable-msc4133")]
impl Serialize for CustomProfileFieldValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.field, &self.value)?;
        map.end()
    }
}

/// Helper type to deserialize [`ProfileFieldValue`].
///
/// If the inner value is set, this will try to deserialize a map entry using this key, otherwise
/// this will deserialize the first key-value pair encountered.
#[cfg(feature = "unstable-msc4133")]
struct ProfileFieldValueVisitor(Option<ProfileFieldName>);

#[cfg(feature = "unstable-msc4133")]
impl<'de> Visitor<'de> for ProfileFieldValueVisitor {
    type Value = Option<ProfileFieldValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("enum ProfileFieldValue")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let field = if let Some(field) = self.0 {
            let mut found = false;

            while let Some(key) = map.next_key::<ProfileFieldName>()? {
                if key == field {
                    found = true;
                    break;
                }
            }

            if !found {
                return Ok(None);
            }

            field
        } else {
            let Some(field) = map.next_key()? else {
                return Ok(None);
            };

            field
        };

        Ok(Some(match field {
            ProfileFieldName::AvatarUrl => ProfileFieldValue::AvatarUrl(map.next_value()?),
            ProfileFieldName::DisplayName => ProfileFieldValue::DisplayName(map.next_value()?),
            ProfileFieldName::_Custom(field) => {
                ProfileFieldValue::_Custom(CustomProfileFieldValue {
                    field: field.0.into(),
                    value: map.next_value()?,
                })
            }
        }))
    }
}

#[cfg(feature = "unstable-msc4133")]
fn deserialize_profile_field_value_option<'de, D>(
    deserializer: D,
) -> Result<Option<ProfileFieldValue>, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_map(ProfileFieldValueVisitor(None))
}

#[cfg(feature = "unstable-msc4133")]
impl<'de> Deserialize<'de> for ProfileFieldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserialize_profile_field_value_option(deserializer)?
            .ok_or_else(|| de::Error::invalid_length(0, &"at least one key-value pair"))
    }
}

#[cfg(all(test, feature = "unstable-msc4133"))]
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
