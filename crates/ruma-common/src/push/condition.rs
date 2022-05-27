use std::{collections::BTreeMap, ops::RangeBounds, str::FromStr};

use js_int::{Int, UInt};
use serde::{Deserialize, Serialize};
use serde_json::{to_value as to_json_value, value::Value as JsonValue};
use tracing::{instrument, warn};
use wildmatch::WildMatch;

use crate::{power_levels::NotificationPowerLevels, serde::Raw, OwnedRoomId, OwnedUserId, UserId};

mod room_member_count_is;

pub use room_member_count_is::{ComparisonOperator, RoomMemberCountIs};

/// A condition that must apply for an associated push rule's action to be taken.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PushCondition {
    /// A glob pattern match on a field of the event.
    EventMatch {
        /// The dot-separated field of the event to match.
        key: String,

        /// The glob-style pattern to match against.
        ///
        /// Patterns with no special glob characters should be treated as having asterisks
        /// prepended and appended when testing the condition.
        pattern: String,
    },

    /// Matches unencrypted messages where `content.body` contains the owner's display name in that
    /// room.
    ContainsDisplayName,

    /// Matches the current number of members in the room.
    RoomMemberCount {
        /// The condition on the current number of members in the room.
        is: RoomMemberCountIs,
    },

    /// Takes into account the current power levels in the room, ensuring the sender of the event
    /// has high enough power to trigger the notification.
    SenderNotificationPermission {
        /// The field in the power level event the user needs a minimum power level for.
        ///
        /// Fields must be specified under the `notifications` property in the power level event's
        /// `content`.
        key: String,
    },
}

pub(super) fn check_event_match(
    event: &FlattenedJson,
    key: &str,
    pattern: &str,
    context: &PushConditionRoomCtx,
) -> bool {
    let value = match key {
        "room_id" => context.room_id.as_str(),
        _ => match event.get(key) {
            Some(v) => v,
            None => return false,
        },
    };

    value.matches_pattern(pattern, key == "content.body")
}

impl PushCondition {
    /// Check if this condition applies to the event.
    ///
    /// # Arguments
    ///
    /// * `event` - The flattened JSON representation of a room message event.
    /// * `context` - The context of the room at the time of the event.
    pub fn applies(&self, event: &FlattenedJson, context: &PushConditionRoomCtx) -> bool {
        if event.get("sender").map_or(false, |sender| sender == context.user_id) {
            return false;
        }

        match self {
            Self::EventMatch { key, pattern } => check_event_match(event, key, pattern, context),
            Self::ContainsDisplayName => {
                let value = match event.get("content.body") {
                    Some(v) => v,
                    None => return false,
                };

                value.matches_pattern(&context.user_display_name, true)
            }
            Self::RoomMemberCount { is } => is.contains(&context.member_count),
            Self::SenderNotificationPermission { key } => {
                let sender_id = match event.get("sender") {
                    Some(v) => match <&UserId>::try_from(v) {
                        Ok(u) => u,
                        Err(_) => return false,
                    },
                    None => return false,
                };

                let sender_level = context
                    .users_power_levels
                    .get(sender_id)
                    .unwrap_or(&context.default_power_level);

                match context.notification_power_levels.get(key) {
                    Some(l) => sender_level >= l,
                    None => false,
                }
            }
        }
    }
}

/// The context of the room associated to an event to be able to test all push conditions.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PushConditionRoomCtx {
    /// The ID of the room.
    pub room_id: OwnedRoomId,

    /// The number of members in the room.
    pub member_count: UInt,

    /// The users matrix ID.
    pub user_id: OwnedUserId,

    /// The display name of the current user in the room.
    pub user_display_name: String,

    /// The power levels of the users of the room.
    pub users_power_levels: BTreeMap<OwnedUserId, Int>,

    /// The default power level of the users of the room.
    pub default_power_level: Int,

    /// The notification power levels of the room.
    pub notification_power_levels: NotificationPowerLevels,
}

/// Additional functions for character matching.
trait CharExt {
    /// Whether or not this char can be part of a word.
    fn is_word_char(&self) -> bool;
}

impl CharExt for char {
    fn is_word_char(&self) -> bool {
        self.is_alphanumeric() || *self == '_'
    }
}

/// Additional functions for string matching.
trait StrExt {
    /// Get the length of the char at `index`. The byte index must correspond to
    /// the start of a char boundary.
    fn char_len(&self, index: usize) -> usize;

    /// Get the char at `index`. The byte index must correspond to the start of
    /// a char boundary.
    fn char_at(&self, index: usize) -> char;

    /// Get the index of the char that is before the char at `index`. The byte index
    /// must correspond to a char boundary.
    ///
    /// Returns `None` if there's no previous char. Otherwise, returns the char.
    fn find_prev_char(&self, index: usize) -> Option<char>;

    /// Matches this string against `pattern`.
    ///
    /// The match is case insensitive.
    ///
    /// If `match_words` is `true`, looks for `pattern` as a substring of `self`,
    /// and checks that it is separated from other words. Otherwise, checks
    /// `pattern` as a glob with wildcards `*` and `?`.
    fn matches_pattern(&self, pattern: &str, match_words: bool) -> bool;

    /// Matches this string against `pattern`, with word boundaries.
    ///
    /// The match is case sensitive.
    fn matches_word(&self, pattern: &str) -> bool;
}

impl StrExt for str {
    fn char_len(&self, index: usize) -> usize {
        let mut len = 1;
        while !self.is_char_boundary(index + len) {
            len += 1;
        }
        len
    }

    fn char_at(&self, index: usize) -> char {
        let end = index + self.char_len(index);
        let char_str = &self[index..end];
        char::from_str(char_str)
            .unwrap_or_else(|_| panic!("Could not convert str '{}' to char", char_str))
    }

    fn find_prev_char(&self, index: usize) -> Option<char> {
        if index == 0 {
            return None;
        }

        let mut pos = index - 1;
        while !self.is_char_boundary(pos) {
            pos -= 1;
        }
        Some(self.char_at(pos))
    }

    fn matches_pattern(&self, pattern: &str, match_words: bool) -> bool {
        let value = &self.to_lowercase();
        let pattern = &pattern.to_lowercase();

        if match_words {
            value.matches_word(pattern)
        } else {
            WildMatch::new(pattern).matches(value)
        }
    }

    fn matches_word(&self, pattern: &str) -> bool {
        if self == pattern {
            return true;
        }
        if pattern.is_empty() {
            return false;
        }

        match self.find(pattern) {
            Some(start) => {
                let end = start + pattern.len();

                // Look if the match has word boundaries.
                let word_boundary_start = !self.char_at(start).is_word_char()
                    || self.find_prev_char(start).map_or(true, |c| !c.is_word_char());

                if word_boundary_start {
                    let word_boundary_end = end == self.len()
                        || !self.find_prev_char(end).unwrap().is_word_char()
                        || !self.char_at(end).is_word_char();

                    if word_boundary_end {
                        return true;
                    }
                }

                // Find next word.
                let non_word_str = &self[start..];
                let non_word = match non_word_str.find(|c: char| !c.is_word_char()) {
                    Some(pos) => pos,
                    None => return false,
                };

                let word_str = &non_word_str[non_word..];
                let word = match word_str.find(|c: char| c.is_word_char()) {
                    Some(pos) => pos,
                    None => return false,
                };

                word_str[word..].matches_word(pattern)
            }
            None => false,
        }
    }
}

/// The flattened representation of a JSON object.
#[derive(Clone, Debug)]
pub struct FlattenedJson {
    /// The internal map containing the flattened JSON as a pair path, value.
    map: BTreeMap<String, String>,
}

impl FlattenedJson {
    /// Create a `FlattenedJson` from `Raw`.
    pub fn from_raw<T>(raw: &Raw<T>) -> Self {
        let mut s = Self { map: BTreeMap::new() };
        s.flatten_value(to_json_value(raw).unwrap(), "".into());
        s
    }

    /// Flatten and insert the `value` at `path`.
    #[instrument(skip(self, value))]
    fn flatten_value(&mut self, value: JsonValue, path: String) {
        match value {
            JsonValue::Object(fields) => {
                for (key, value) in fields {
                    let path = if path.is_empty() { key } else { format!("{}.{}", path, key) };
                    self.flatten_value(value, path);
                }
            }
            JsonValue::String(s) => {
                if self.map.insert(path.clone(), s).is_some() {
                    warn!("Duplicate path in flattened JSON: {}", path);
                }
            }
            JsonValue::Number(_) | JsonValue::Bool(_) | JsonValue::Array(_) | JsonValue::Null => {}
        }
    }

    /// Value associated with the given `path`.
    pub fn get(&self, path: &str) -> Option<&str> {
        self.map.get(path).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{room_id, serde::Raw, user_id};
    use assert_matches::assert_matches;
    use js_int::{int, uint};
    use maplit::btreemap;
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{FlattenedJson, PushCondition, PushConditionRoomCtx, RoomMemberCountIs, StrExt};
    use crate::power_levels::NotificationPowerLevels;

    #[test]
    fn serialize_event_match_condition() {
        let json_data = json!({
            "key": "content.msgtype",
            "kind": "event_match",
            "pattern": "m.notice"
        });
        assert_eq!(
            to_json_value(&PushCondition::EventMatch {
                key: "content.msgtype".into(),
                pattern: "m.notice".into(),
            })
            .unwrap(),
            json_data
        );
    }

    #[test]
    fn serialize_contains_display_name_condition() {
        assert_eq!(
            to_json_value(&PushCondition::ContainsDisplayName).unwrap(),
            json!({ "kind": "contains_display_name" })
        );
    }

    #[test]
    fn serialize_room_member_count_condition() {
        let json_data = json!({
            "is": "2",
            "kind": "room_member_count"
        });
        assert_eq!(
            to_json_value(&PushCondition::RoomMemberCount {
                is: RoomMemberCountIs::from(uint!(2))
            })
            .unwrap(),
            json_data
        );
    }

    #[test]
    fn serialize_sender_notification_permission_condition() {
        let json_data = json!({
            "key": "room",
            "kind": "sender_notification_permission"
        });
        assert_eq!(
            json_data,
            to_json_value(&PushCondition::SenderNotificationPermission { key: "room".into() })
                .unwrap()
        );
    }

    #[test]
    fn deserialize_event_match_condition() {
        let json_data = json!({
            "key": "content.msgtype",
            "kind": "event_match",
            "pattern": "m.notice"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::EventMatch { key, pattern }
            if key == "content.msgtype" && pattern == "m.notice"
        );
    }

    #[test]
    fn deserialize_contains_display_name_condition() {
        assert_matches!(
            from_json_value::<PushCondition>(json!({ "kind": "contains_display_name" })).unwrap(),
            PushCondition::ContainsDisplayName
        );
    }

    #[test]
    fn deserialize_room_member_count_condition() {
        let json_data = json!({
            "is": "2",
            "kind": "room_member_count"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::RoomMemberCount { is }
            if is == RoomMemberCountIs::from(uint!(2))
        );
    }

    #[test]
    fn deserialize_sender_notification_permission_condition() {
        let json_data = json!({
            "key": "room",
            "kind": "sender_notification_permission"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::SenderNotificationPermission {
                key
            } if key == "room"
        );
    }

    #[test]
    fn words_match() {
        assert!("foo bar".matches_word("foo"));
        assert!(!"Foo bar".matches_word("foo"));
        assert!(!"foobar".matches_word("foo"));
        assert!("foobar foo".matches_word("foo"));
        assert!(!"foobar foobar".matches_word("foo"));
        assert!(!"foobar bar".matches_word("bar bar"));
        assert!("foobar bar bar".matches_word("bar bar"));
        assert!(!"foobar bar barfoo".matches_word("bar bar"));
        assert!("conduit ⚡️".matches_word("conduit ⚡️"));
        assert!("conduit ⚡️".matches_word("conduit"));
        assert!("conduit ⚡️".matches_word("⚡️"));
        assert!("conduit⚡️".matches_word("conduit"));
        assert!("conduit⚡️".matches_word("⚡️"));
        assert!("⚡️conduit".matches_word("conduit"));
        assert!("⚡️conduit".matches_word("⚡️"));
        assert!("Ruma Dev👩‍💻".matches_word("Dev"));
        assert!("Ruma Dev👩‍💻".matches_word("👩‍💻"));
        assert!("Ruma Dev👩‍💻".matches_word("Dev👩‍💻"));
    }

    #[test]
    fn patterns_match() {
        // Word matching
        assert!("foo bar".matches_pattern("foo", true));
        assert!("Foo bar".matches_pattern("foo", true));
        assert!(!"foobar".matches_pattern("foo", true));
        assert!(!"foo bar".matches_pattern("foo*", true));
        assert!("".matches_pattern("", true));
        assert!(!"foo".matches_pattern("", true));

        // Glob matching
        assert!(!"foo bar".matches_pattern("foo", false));
        assert!("foo".matches_pattern("foo", false));
        assert!("foo".matches_pattern("foo*", false));
        assert!("foobar".matches_pattern("foo*", false));
        assert!("foo bar".matches_pattern("foo*", false));
        assert!(!"foo".matches_pattern("foo?", false));
        assert!("fooo".matches_pattern("foo?", false));
        assert!("FOO".matches_pattern("foo", false));
        assert!("".matches_pattern("", false));
        assert!("".matches_pattern("*", false));
        assert!(!"foo".matches_pattern("", false));
    }

    #[test]
    fn conditions_apply_to_events() {
        let first_sender = user_id!("@worthy_whale:server.name").to_owned();

        let mut users_power_levels = BTreeMap::new();
        users_power_levels.insert(first_sender, int!(25));

        let context = PushConditionRoomCtx {
            room_id: room_id!("!room:server.name").to_owned(),
            member_count: uint!(3),
            user_id: user_id!("@gorilla:server.name").to_owned(),
            user_display_name: "Groovy Gorilla".into(),
            users_power_levels,
            default_power_level: int!(50),
            notification_power_levels: NotificationPowerLevels { room: int!(50) },
        };

        let first_event_raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@worthy_whale:server.name",
                "content": {
                    "msgtype": "m.text",
                    "body": "@room Give a warm welcome to Groovy Gorilla"
                }
            }"#,
        )
        .unwrap();
        let first_event = FlattenedJson::from_raw(&first_event_raw);

        let second_event_raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@party_bot:server.name",
                "content": {
                    "msgtype": "m.notice",
                    "body": "@room Ready to come to the party?"
                }
            }"#,
        )
        .unwrap();
        let second_event = FlattenedJson::from_raw(&second_event_raw);

        let correct_room = PushCondition::EventMatch {
            key: "room_id".into(),
            pattern: "!room:server.name".into(),
        };
        let incorrect_room = PushCondition::EventMatch {
            key: "room_id".into(),
            pattern: "!incorrect:server.name".into(),
        };

        assert!(correct_room.applies(&first_event, &context));
        assert!(!incorrect_room.applies(&first_event, &context));

        let keyword =
            PushCondition::EventMatch { key: "content.body".into(), pattern: "come".into() };

        assert!(!keyword.applies(&first_event, &context));
        assert!(keyword.applies(&second_event, &context));

        let msgtype =
            PushCondition::EventMatch { key: "content.msgtype".into(), pattern: "m.notice".into() };

        assert!(!msgtype.applies(&first_event, &context));
        assert!(msgtype.applies(&second_event, &context));

        let member_count_eq =
            PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(3)) };
        let member_count_gt =
            PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(2)..) };
        let member_count_lt =
            PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(..uint!(3)) };

        assert!(member_count_eq.applies(&first_event, &context));
        assert!(member_count_gt.applies(&first_event, &context));
        assert!(!member_count_lt.applies(&first_event, &context));

        let contains_display_name = PushCondition::ContainsDisplayName;

        assert!(contains_display_name.applies(&first_event, &context));
        assert!(!contains_display_name.applies(&second_event, &context));

        let sender_notification_permission =
            PushCondition::SenderNotificationPermission { key: "room".into() };

        assert!(!sender_notification_permission.applies(&first_event, &context));
        assert!(sender_notification_permission.applies(&second_event, &context));
    }

    #[test]
    fn flattened_json_values() {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "string": "Hello World",
                "number": 10,
                "array": [1, 2],
                "boolean": true,
                "null": null
            }"#,
        )
        .unwrap();

        let flattened = FlattenedJson::from_raw(&raw);
        assert_eq!(flattened.map, btreemap! { "string".into() => "Hello World".into() });
    }

    #[test]
    fn flattened_json_nested() {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "desc": "Level 0",
                "up": {
                    "desc": "Level 1",
                    "up": {
                        "desc": "Level 2"
                    }
                }
            }"#,
        )
        .unwrap();

        let flattened = FlattenedJson::from_raw(&raw);
        assert_eq!(
            flattened.map,
            btreemap! {
                "desc".into() => "Level 0".into(),
                "up.desc".into() => "Level 1".into(),
                "up.up.desc".into() => "Level 2".into(),
            },
        );
    }
}
