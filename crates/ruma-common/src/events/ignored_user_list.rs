//! Types for the [`m.ignored_user_list`] event.
//!
//! [`m.ignored_user_list`]: https://spec.matrix.org/v1.4/client-server-api/#mignored_user_list

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::OwnedUserId;

/// The content of an `m.ignored_user_list` event.
///
/// A list of users to ignore.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.ignored_user_list", kind = GlobalAccountData)]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    #[serde(with = "vec_as_map_of_empty")]
    pub ignored_users: Vec<OwnedUserId>,
}

impl IgnoredUserListEventContent {
    /// Creates a new `IgnoredUserListEventContent` from the given user IDs.
    pub fn new(ignored_users: Vec<OwnedUserId>) -> Self {
        Self { ignored_users }
    }
}

mod vec_as_map_of_empty {
    use std::{fmt, marker::PhantomData};

    use serde::{
        de::{self, Deserialize, Deserializer},
        ser::{SerializeMap, Serializer},
        Serialize,
    };

    /// Serialize the given `Vec<T>` as a map of `T => Empty`.
    #[allow(clippy::ptr_arg)]
    pub fn serialize<S, T>(vec: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + Eq + Ord,
    {
        let mut map = serializer.serialize_map(Some(vec.len()))?;
        for item in vec {
            map.serialize_entry(item, &Empty {})?;
        }
        map.end()
    }

    /// Deserialize an object and return the keys as a `Vec<T>`.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Eq + Ord,
    {
        struct MapOfEmptyVisitor<T>(PhantomData<T>);
        impl<'de, T> de::Visitor<'de> for MapOfEmptyVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Vec<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "an object/map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut items = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((item, _)) = map.next_entry::<T, Empty>()? {
                    items.push(item);
                }
                Ok(items)
            }
        }

        deserializer.deserialize_map(MapOfEmptyVisitor(PhantomData))
    }

    #[derive(Clone, Debug, Serialize)]
    struct Empty {}

    impl<'de> Deserialize<'de> for Empty {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct EmptyMapVisitor;

            impl<'de> de::Visitor<'de> for EmptyMapVisitor {
                type Value = Empty;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "an object/map")
                }

                fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
                where
                    A: de::MapAccess<'de>,
                {
                    Ok(Empty {})
                }
            }

            deserializer.deserialize_map(EmptyMapVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::IgnoredUserListEventContent;
    use crate::{
        events::{AnyGlobalAccountDataEvent, GlobalAccountDataEvent},
        user_id,
    };

    #[test]
    fn serialization() {
        let ignored_user_list_event = GlobalAccountDataEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![user_id!("@carl:example.com").to_owned()],
            },
        };

        let json = json!({
            "content": {
                "ignored_users": {
                    "@carl:example.com": {}
                }
            },
            "type": "m.ignored_user_list"
        });

        assert_eq!(to_json_value(ignored_user_list_event).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {
                "ignored_users": {
                    "@carl:example.com": {}
                }
            },
            "type": "m.ignored_user_list"
        });

        let ev = assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::IgnoredUserList(ev)) => ev
        );
        assert_eq!(ev.content.ignored_users, vec![user_id!("@carl:example.com")]);
    }
}
