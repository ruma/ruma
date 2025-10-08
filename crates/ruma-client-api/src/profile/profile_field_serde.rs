use std::{borrow::Cow, fmt, marker::PhantomData};

use serde::{de, ser::SerializeMap, Deserialize, Serialize, Serializer};

use super::{CustomProfileFieldValue, ProfileFieldName, ProfileFieldValue, StaticProfileField};

/// Helper type to deserialize any type that implements [`StaticProfileField`].
pub(super) struct StaticProfileFieldVisitor<F: StaticProfileField>(pub(super) PhantomData<F>);

impl<'de, F: StaticProfileField> de::Visitor<'de> for StaticProfileFieldVisitor<F> {
    type Value = Option<F::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a map with optional key `{}` and value", F::NAME)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
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

impl Serialize for CustomProfileFieldValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
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
pub(super) struct ProfileFieldValueVisitor(pub(super) Option<ProfileFieldName>);

impl<'de> de::Visitor<'de> for ProfileFieldValueVisitor {
    type Value = Option<ProfileFieldValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("enum ProfileFieldValue")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
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
            ProfileFieldName::TimeZone => ProfileFieldValue::TimeZone(map.next_value()?),
            ProfileFieldName::_Custom(field) => {
                ProfileFieldValue::_Custom(CustomProfileFieldValue {
                    field: field.0.into(),
                    value: map.next_value()?,
                })
            }
        }))
    }
}

pub(super) fn deserialize_profile_field_value_option<'de, D>(
    deserializer: D,
) -> Result<Option<ProfileFieldValue>, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_map(ProfileFieldValueVisitor(None))
}

impl<'de> Deserialize<'de> for ProfileFieldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserialize_profile_field_value_option(deserializer)?
            .ok_or_else(|| de::Error::invalid_length(0, &"at least one key-value pair"))
    }
}
