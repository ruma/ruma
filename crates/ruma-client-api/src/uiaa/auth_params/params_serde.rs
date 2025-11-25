//! Custom Serialize / Deserialize implementations for the authentication parameters types.

use std::{collections::BTreeMap, fmt};

use serde::{
    Deserialize, Deserializer,
    de::{self, Error},
};

use super::{PolicyDefinition, PolicyTranslation};

// Custom implementation because the translations are at the root of the object, but we want to
// ignore fields whose value fails to deserialize because they might be custom fields.
impl<'de> Deserialize<'de> for PolicyDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PolicyDefinitionVisitor;

        impl<'de> de::Visitor<'de> for PolicyDefinitionVisitor {
            type Value = PolicyDefinition;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("PolicyDefinition")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut version = None;
                let mut translations = BTreeMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    if key == "version" {
                        if version.is_some() {
                            return Err(A::Error::duplicate_field("version"));
                        }

                        version = Some(map.next_value()?);
                        continue;
                    }

                    if let Ok(translation) = map.next_value::<PolicyTranslation>() {
                        if translations.contains_key(&key) {
                            return Err(A::Error::custom(format!("duplicate field `{key}`")));
                        }

                        translations.insert(key, translation);
                    }
                }

                Ok(PolicyDefinition {
                    version: version.ok_or_else(|| A::Error::missing_field("version"))?,
                    translations,
                })
            }
        }

        deserializer.deserialize_map(PolicyDefinitionVisitor)
    }
}
