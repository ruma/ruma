//! Types for the [`m.tag`] event.
//!
//! [`m.tag`]: https://spec.matrix.org/v1.2/client-server-api/#mtag

use std::{collections::BTreeMap, error::Error, fmt, str::FromStr};

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{serde::deserialize_cow_str, PrivOwnedStr};

/// Map of tag names to tag info.
pub type Tags = BTreeMap<TagName, TagInfo>;

/// The content of an `m.tag` event.
///
/// Informs the client of tags on a room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.tag", kind = RoomAccountData)]
pub struct TagEventContent {
    /// A map of tag names to tag info.
    pub tags: Tags,
}

impl TagEventContent {
    /// Creates a new `TagEventContent` with the given `Tags`.
    pub fn new(tags: Tags) -> Self {
        Self { tags }
    }
}

impl From<Tags> for TagEventContent {
    fn from(tags: Tags) -> Self {
        Self::new(tags)
    }
}

/// A user-defined tag name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserTagName {
    name: String,
}

impl AsRef<str> for UserTagName {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl FromStr for UserTagName {
    type Err = InvalidUserTagName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("u.") {
            Ok(Self { name: s.into() })
        } else {
            Err(InvalidUserTagName)
        }
    }
}

/// An error returned when attempting to create a UserTagName with a string that would make it
/// invalid.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct InvalidUserTagName;

impl fmt::Display for InvalidUserTagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing 'u.' prefix in UserTagName")
    }
}

impl Error for InvalidUserTagName {}

/// The name of a tag.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum TagName {
    /// `m.favourite`: The user's favorite rooms.
    ///
    /// These should be shown with higher precedence than other rooms.
    Favorite,

    /// `m.lowpriority`: These should be shown with lower precedence than others.
    LowPriority,

    /// `m.server_notice`: Used to identify
    /// [Server Notice Rooms](https://spec.matrix.org/v1.2/client-server-api/#server-notices).
    ServerNotice,

    /// `u.*`: User-defined tag
    User(UserTagName),

    /// A custom tag
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl TagName {
    /// Returns the display name of the tag.
    ///
    /// That means the string after `m.` or `u.` for spec- and user-defined tag names, and the
    /// string after the last dot for custom tags. If no dot is found, returns the whole string.
    pub fn display_name(&self) -> &str {
        match self {
            Self::_Custom(s) => {
                let start = s.0.rfind('.').map(|p| p + 1).unwrap_or(0);
                &self.as_ref()[start..]
            }
            _ => &self.as_ref()[2..],
        }
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        match self {
            Self::Favorite => "m.favourite",
            Self::LowPriority => "m.lowpriority",
            Self::ServerNotice => "m.server_notice",
            Self::User(tag) => tag.as_ref(),
            Self::_Custom(s) => &s.0,
        }
    }
}

impl<T> From<T> for TagName
where
    T: AsRef<str> + Into<String>,
{
    fn from(s: T) -> TagName {
        match s.as_ref() {
            "m.favourite" => Self::Favorite,
            "m.lowpriority" => Self::LowPriority,
            "m.server_notice" => Self::ServerNotice,
            s if s.starts_with("u.") => Self::User(UserTagName { name: s.into() }),
            s => Self::_Custom(PrivOwnedStr(s.into())),
        }
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let cow = deserialize_cow_str(deserializer)?;
        Ok(cow.into())
    }
}

impl Serialize for TagName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

/// Information about a tag.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct TagInfo {
    /// Value to use for lexicographically ordering rooms with this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<f64>,
}

impl TagInfo {
    /// Creates an empty `TagInfo`.
    pub fn new() -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use serde_json::{json, to_value as to_json_value};

    use super::{TagEventContent, TagInfo, TagName};

    #[test]
    fn serialization() {
        let tags = btreemap! {
            TagName::Favorite => TagInfo::new(),
            TagName::LowPriority => TagInfo::new(),
            TagName::ServerNotice => TagInfo::new(),
            "u.custom".to_owned().into() => TagInfo { order: Some(0.9) }
        };

        let content = TagEventContent { tags };

        assert_eq!(
            to_json_value(content).unwrap(),
            json!({
                "tags": {
                    "m.favourite": {},
                    "m.lowpriority": {},
                    "m.server_notice": {},
                    "u.custom": {
                        "order": 0.9
                    }
                },
            })
        );
    }

    #[test]
    fn display_name() {
        assert_eq!(TagName::Favorite.display_name(), "favourite");
        assert_eq!(TagName::LowPriority.display_name(), "lowpriority");
        assert_eq!(TagName::ServerNotice.display_name(), "server_notice");
        assert_eq!(TagName::from("u.Work").display_name(), "Work");
        assert_eq!(TagName::from("rs.conduit.rules").display_name(), "rules");
        assert_eq!(TagName::from("Play").display_name(), "Play");
    }
}
