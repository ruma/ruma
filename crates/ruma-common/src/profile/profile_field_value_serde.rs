use std::fmt;

use serde::{Deserialize, Serialize, Serializer, de, ser::SerializeMap};

use super::{CustomProfileFieldValue, ProfileFieldName, ProfileFieldValue};

/// Helper type to deserialize [`ProfileFieldValue`].
///
/// If the field name is set, this will try to deserialize a map entry using this key, otherwise
/// this will deserialize the first key-value pair encountered.
pub struct ProfileFieldValueVisitor(Option<ProfileFieldName>);

impl ProfileFieldValueVisitor {
    /// Construct a `ProfileFieldValueVisitor` for the given optional field name.
    pub fn new(field: Option<ProfileFieldName>) -> Self {
        Self(field)
    }

    /// Try to find the key in the map matching the proper field name if it is set, or return the
    /// first key if it is not set.
    ///
    /// Returns `Ok(Some(_))` if the field name was found, `Ok(None)` if it wasn't found, and
    /// `Err(_)` if deserialization of a key failed.
    fn find_field_name<'de, V>(self, map: &mut V) -> Result<Option<ProfileFieldName>, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<ProfileFieldName>()? {
            if self.0.as_ref().is_none_or(|field| key == *field) {
                return Ok(Some(key));
            }
        }

        Ok(None)
    }
}

impl<'de> de::Visitor<'de> for ProfileFieldValueVisitor {
    type Value = Option<ProfileFieldValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("enum ProfileFieldValue")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let Some(field) = self.find_field_name(&mut map)? else {
            return Ok(None);
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

impl<'de> Deserialize<'de> for ProfileFieldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer
            .deserialize_map(ProfileFieldValueVisitor(None))?
            .ok_or_else(|| de::Error::invalid_length(0, &"at least one key-value pair"))
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
