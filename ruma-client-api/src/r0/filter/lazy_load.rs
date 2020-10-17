use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeStruct as _, Serializer},
};

/// Specifies options for [lazy-loading membership events][lazy-loading] on
/// supported endpoints
///
/// [lazy-loading]: https://matrix.org/docs/spec/client_server/r0.6.0#lazy-loading-room-members
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LazyLoadOptions {
    /// Disables lazy-loading of membership events.
    Disabled,

    /// Enables lazy-loading of events.
    Enabled {
        /// If `true`, sends all membership events for all events, even if they have
        /// already been sent to the client. Defaults to `false`.
        include_redundant_members: bool,
    },
}

impl Serialize for LazyLoadOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state;
        match *self {
            Self::Enabled { include_redundant_members: true } => {
                state = serializer.serialize_struct("LazyLoad", 2)?;
                state.serialize_field("lazy_load_members", &true)?;
                state.serialize_field("include_redundant_members", &true)?;
            }
            Self::Enabled { .. } => {
                state = serializer.serialize_struct("LazyLoad", 1)?;
                state.serialize_field("lazy_load_members", &true)?;
            }
            _ => {
                state = serializer.serialize_struct("LazyLoad", 0)?;
            }
        }
        state.end()
    }
}

impl Default for LazyLoadOptions {
    fn default() -> Self {
        Self::Disabled
    }
}

struct LazyLoadOptionsVisitor;

impl<'de> Visitor<'de> for LazyLoadOptionsVisitor {
    type Value = LazyLoadOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Lazy load options")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut lazy_load_members = false;
        let mut include_redundant_members = false;
        while let Some((key, value)) = access.next_entry::<String, bool>()? {
            match &*key {
                "lazy_load_members" => lazy_load_members = value,
                "include_redundant_members" => include_redundant_members = value,
                _ => {}
            };
        }

        Ok(if lazy_load_members {
            LazyLoadOptions::Enabled { include_redundant_members }
        } else {
            LazyLoadOptions::Disabled
        })
    }
}

impl<'de> Deserialize<'de> for LazyLoadOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LazyLoadOptionsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::LazyLoadOptions;

    #[test]
    fn serialize_disabled() {
        let lazy_load_options = LazyLoadOptions::Disabled;
        assert_eq!(to_json_value(lazy_load_options).unwrap(), json!({}));
    }

    #[test]
    fn serialize_no_redundant() {
        let lazy_load_options = LazyLoadOptions::Enabled { include_redundant_members: false };
        assert_eq!(to_json_value(lazy_load_options).unwrap(), json!({ "lazy_load_members": true }));
    }

    #[test]
    fn serialize_with_redundant() {
        let lazy_load_options = LazyLoadOptions::Enabled { include_redundant_members: true };
        assert_eq!(
            to_json_value(lazy_load_options).unwrap(),
            json!({ "lazy_load_members": true, "include_redundant_members": true })
        );
    }

    #[test]
    fn deserialize_no_lazy_load() {
        let json = json!({});
        assert_eq!(from_json_value::<LazyLoadOptions>(json).unwrap(), LazyLoadOptions::Disabled);

        let json = json!({ "include_redundant_members": true });
        assert_eq!(from_json_value::<LazyLoadOptions>(json).unwrap(), LazyLoadOptions::Disabled);
    }
}
