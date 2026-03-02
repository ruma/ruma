use std::fmt;

use serde::de;

/// Helper type to deserialize any type that implements [`StaticProfileField`].
pub(super) struct StaticProfileFieldVisitor<F: super::StaticProfileField>(
    pub(super) std::marker::PhantomData<F>,
);

impl<'de, F: super::StaticProfileField> de::Visitor<'de> for StaticProfileFieldVisitor<F> {
    type Value = Option<F::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a map with optional key `{}` and value", F::NAME)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        use std::borrow::Cow;

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
