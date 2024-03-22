use std::{collections::BTreeMap, ops::RangeBounds, str::FromStr};

use js_int::{Int, UInt};
use regex::bytes::Regex;
#[cfg(feature = "unstable-msc3931")]
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};
use serde_json::value::Value as JsonValue;
use wildmatch::WildMatch;

use crate::{power_levels::NotificationPowerLevels, OwnedRoomId, OwnedUserId, UserId};
#[cfg(feature = "unstable-msc3931")]
use crate::{PrivOwnedStr, RoomVersionId};

mod flattened_json;
mod push_condition_serde;
mod room_member_count_is;

pub use self::{
    flattened_json::{FlattenedJson, FlattenedJsonValue, ScalarJsonValue},
    room_member_count_is::{ComparisonOperator, RoomMemberCountIs},
};

/// Features supported by room versions.
#[cfg(feature = "unstable-msc3931")]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RoomVersionFeature {
    /// m.extensible_events
    ///
    /// The room supports [extensible events].
    ///
    /// [extensible events]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
    #[cfg(feature = "unstable-msc3932")]
    #[ruma_enum(rename = "org.matrix.msc3932.extensible_events")]
    ExtensibleEvents,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(feature = "unstable-msc3931")]
impl RoomVersionFeature {
    /// Get the default features for the given room version.
    pub fn list_for_room_version(version: &RoomVersionId) -> Vec<Self> {
        match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10
            | RoomVersionId::V11
            | RoomVersionId::_Custom(_) => vec![],
        }
    }
}

/// A condition that must apply for an associated push rule's action to be taken.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum PushCondition {
    /// A glob pattern match on a field of the event.
    EventMatch {
        /// The [dot-separated path] of the property of the event to match.
        ///
        /// [dot-separated path]: https://spec.matrix.org/latest/appendices/#dot-separated-property-paths
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

    /// Apply the rule only to rooms that support a given feature.
    #[cfg(feature = "unstable-msc3931")]
    RoomVersionSupports {
        /// The feature the room must support for the push rule to apply.
        feature: RoomVersionFeature,
    },

    /// Exact value match on a property of the event.
    EventPropertyIs {
        /// The [dot-separated path] of the property of the event to match.
        ///
        /// [dot-separated path]: https://spec.matrix.org/latest/appendices/#dot-separated-property-paths
        key: String,

        /// The value to match against.
        value: ScalarJsonValue,
    },

    /// Exact value match on a value in an array property of the event.
    EventPropertyContains {
        /// The [dot-separated path] of the property of the event to match.
        ///
        /// [dot-separated path]: https://spec.matrix.org/latest/appendices/#dot-separated-property-paths
        key: String,

        /// The value to match against.
        value: ScalarJsonValue,
    },

    #[doc(hidden)]
    _Custom(_CustomPushCondition),
}

pub(super) fn check_event_match(
    event: &FlattenedJson,
    key: &str,
    pattern: &str,
    context: &PushConditionRoomCtx,
) -> bool {
    let value = match key {
        "room_id" => context.room_id.as_str(),
        _ => match event.get_str(key) {
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
    /// * `context` - The context of the room at the time of the event. If the power levels context
    ///   is missing from it, conditions that depend on it will never apply.
    pub fn applies(&self, event: &FlattenedJson, context: &PushConditionRoomCtx) -> bool {
        if event.get_str("sender").is_some_and(|sender| sender == context.user_id) {
            return false;
        }

        match self {
            Self::EventMatch { key, pattern } => check_event_match(event, key, pattern, context),
            Self::ContainsDisplayName => {
                let value = match event.get_str("content.body") {
                    Some(v) => v,
                    None => return false,
                };

                value.matches_pattern(&context.user_display_name, true)
            }
            Self::RoomMemberCount { is } => is.contains(&context.member_count),
            Self::SenderNotificationPermission { key } => {
                let Some(power_levels) = &context.power_levels else {
                    return false;
                };

                let sender_id = match event.get_str("sender") {
                    Some(v) => match <&UserId>::try_from(v) {
                        Ok(u) => u,
                        Err(_) => return false,
                    },
                    None => return false,
                };

                let sender_level =
                    power_levels.users.get(sender_id).unwrap_or(&power_levels.users_default);

                match power_levels.notifications.get(key) {
                    Some(l) => sender_level >= l,
                    None => false,
                }
            }
            #[cfg(feature = "unstable-msc3931")]
            Self::RoomVersionSupports { feature } => match feature {
                RoomVersionFeature::ExtensibleEvents => {
                    context.supported_features.contains(&RoomVersionFeature::ExtensibleEvents)
                }
                RoomVersionFeature::_Custom(_) => false,
            },
            Self::EventPropertyIs { key, value } => event.get(key).is_some_and(|v| v == value),
            Self::EventPropertyContains { key, value } => event
                .get(key)
                .and_then(FlattenedJsonValue::as_array)
                .is_some_and(|a| a.contains(value)),
            Self::_Custom(_) => false,
        }
    }
}

/// An unknown push condition.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct _CustomPushCondition {
    /// The kind of the condition.
    kind: String,

    /// The additional fields that the condition contains.
    #[serde(flatten)]
    data: BTreeMap<String, JsonValue>,
}

/// The context of the room associated to an event to be able to test all push conditions.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PushConditionRoomCtx {
    /// The ID of the room.
    pub room_id: OwnedRoomId,

    /// The number of members in the room.
    pub member_count: UInt,

    /// The user's matrix ID.
    pub user_id: OwnedUserId,

    /// The display name of the current user in the room.
    pub user_display_name: String,

    /// The room power levels context for the room.
    ///
    /// If this is missing, push rules that require this will never match.
    pub power_levels: Option<PushConditionPowerLevelsCtx>,

    /// The list of features this room's version or the room itself supports.
    #[cfg(feature = "unstable-msc3931")]
    pub supported_features: Vec<RoomVersionFeature>,
}

/// The room power levels context to be able to test the corresponding push conditions.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PushConditionPowerLevelsCtx {
    /// The power levels of the users of the room.
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level of the users of the room.
    pub users_default: Int,

    /// The notification power levels of the room.
    pub notifications: NotificationPowerLevels,
}

/// Additional functions for character matching.
trait CharExt {
    /// Whether or not this char can be part of a word.
    fn is_word_char(&self) -> bool;
}

impl CharExt for char {
    fn is_word_char(&self) -> bool {
        self.is_ascii_alphanumeric() || *self == '_'
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
    /// The pattern can be a glob with wildcards `*` and `?`.
    ///
    /// The match is case insensitive.
    ///
    /// If `match_words` is `true`, checks that the pattern is separated from other words.
    fn matches_pattern(&self, pattern: &str, match_words: bool) -> bool;

    /// Matches this string against `pattern`, with word boundaries.
    ///
    /// The pattern can be a glob with wildcards `*` and `?`.
    ///
    /// A word boundary is defined as the start or end of the value, or any character not in the
    /// sets `[A-Z]`, `[a-z]`, `[0-9]` or `_`.
    ///
    /// The match is case sensitive.
    fn matches_word(&self, pattern: &str) -> bool;

    /// Translate the wildcards in `self` to a regex syntax.
    ///
    /// `self` must only contain wildcards.
    fn wildcards_to_regex(&self) -> String;
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
            .unwrap_or_else(|_| panic!("Could not convert str '{char_str}' to char"))
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

        let has_wildcards = pattern.contains(|c| matches!(c, '?' | '*'));

        if has_wildcards {
            let mut chunks: Vec<String> = vec![];
            let mut prev_wildcard = false;
            let mut chunk_start = 0;

            for (i, c) in pattern.char_indices() {
                if matches!(c, '?' | '*') && !prev_wildcard {
                    if i != 0 {
                        chunks.push(regex::escape(&pattern[chunk_start..i]));
                        chunk_start = i;
                    }

                    prev_wildcard = true;
                } else if prev_wildcard {
                    let chunk = &pattern[chunk_start..i];
                    chunks.push(chunk.wildcards_to_regex());

                    chunk_start = i;
                    prev_wildcard = false;
                }
            }

            let len = pattern.len();
            if !prev_wildcard {
                chunks.push(regex::escape(&pattern[chunk_start..len]));
            } else if prev_wildcard {
                let chunk = &pattern[chunk_start..len];
                chunks.push(chunk.wildcards_to_regex());
            }

            // The word characters in ASCII compatible mode (with the `-u` flag) match the
            // definition in the spec: any character not in the set `[A-Za-z0-9_]`.
            let regex = format!(r"(?-u:^|\W|\b){}(?-u:\b|\W|$)", chunks.concat());
            let re = Regex::new(&regex).expect("regex construction should succeed");
            re.is_match(self.as_bytes())
        } else {
            match self.find(pattern) {
                Some(start) => {
                    let end = start + pattern.len();

                    // Look if the match has word boundaries.
                    let word_boundary_start = !self.char_at(start).is_word_char()
                        || !self.find_prev_char(start).is_some_and(|c| c.is_word_char());

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

    fn wildcards_to_regex(&self) -> String {
        // Simplify pattern to avoid performance issues:
        // - The glob `?**?**?` is equivalent to the glob `???*`
        // - The glob `???*` is equivalent to the regex `.{3,}`
        let question_marks = self.matches('?').count();

        if self.contains('*') {
            format!(".{{{question_marks},}}")
        } else {
            format!(".{{{question_marks}}}")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use js_int::{int, uint};
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{
        FlattenedJson, PushCondition, PushConditionPowerLevelsCtx, PushConditionRoomCtx,
        RoomMemberCountIs, StrExt,
    };
    use crate::{
        owned_room_id, owned_user_id, power_levels::NotificationPowerLevels, serde::Raw,
        OwnedUserId,
    };

    #[test]
    fn serialize_event_match_condition() {
        let json_data = json!({
            "key": "content.msgtype",
            "kind": "event_match",
            "pattern": "m.notice"
        });
        assert_eq!(
            to_json_value(PushCondition::EventMatch {
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
            to_json_value(PushCondition::ContainsDisplayName).unwrap(),
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
            to_json_value(PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(2)) })
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
            to_json_value(PushCondition::SenderNotificationPermission { key: "room".into() })
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
        );
        assert_eq!(key, "content.msgtype");
        assert_eq!(pattern, "m.notice");
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
        );
        assert_eq!(is, RoomMemberCountIs::from(uint!(2)));
    }

    #[test]
    fn deserialize_sender_notification_permission_condition() {
        let json_data = json!({
            "key": "room",
            "kind": "sender_notification_permission"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::SenderNotificationPermission { key }
        );
        assert_eq!(key, "room");
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

        // Regex syntax is escaped
        assert!(!"matrix".matches_word(r"\w*"));
        assert!(r"\w".matches_word(r"\w*"));
        assert!(!"matrix".matches_word("[a-z]*"));
        assert!("[a-z] and [0-9]".matches_word("[a-z]*"));
        assert!(!"m".matches_word("[[:alpha:]]?"));
        assert!("[[:alpha:]]!".matches_word("[[:alpha:]]?"));

        // From the spec: <https://spec.matrix.org/v1.10/client-server-api/#conditions-1>
        assert!("An example event.".matches_word("ex*ple"));
        assert!("exple".matches_word("ex*ple"));
        assert!("An exciting triple-whammy".matches_word("ex*ple"));
    }

    #[test]
    fn patterns_match() {
        // Word matching without glob
        assert!("foo bar".matches_pattern("foo", true));
        assert!("Foo bar".matches_pattern("foo", true));
        assert!(!"foobar".matches_pattern("foo", true));
        assert!("".matches_pattern("", true));
        assert!(!"foo".matches_pattern("", true));
        assert!("foo bar".matches_pattern("foo bar", true));
        assert!(" foo bar ".matches_pattern("foo bar", true));
        assert!("baz foo bar baz".matches_pattern("foo bar", true));
        assert!("foo baré".matches_pattern("foo bar", true));
        assert!(!"bar foo".matches_pattern("foo bar", true));
        assert!("foo bar".matches_pattern("foo ", true));
        assert!("foo ".matches_pattern("foo ", true));
        assert!("foo  ".matches_pattern("foo ", true));
        assert!(" foo  ".matches_pattern("foo ", true));

        // Word matching with glob
        assert!("foo bar".matches_pattern("foo*", true));
        assert!("foo bar".matches_pattern("foo b?r", true));
        assert!(" foo bar ".matches_pattern("foo b?r", true));
        assert!("baz foo bar baz".matches_pattern("foo b?r", true));
        assert!("foo baré".matches_pattern("foo b?r", true));
        assert!(!"bar foo".matches_pattern("foo b?r", true));
        assert!("foo bar".matches_pattern("f*o ", true));
        assert!("foo ".matches_pattern("f*o ", true));
        assert!("foo  ".matches_pattern("f*o ", true));
        assert!(" foo  ".matches_pattern("f*o ", true));

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

        // From the spec: <https://spec.matrix.org/v1.10/client-server-api/#conditions-1>
        assert!("Lunch plans".matches_pattern("lunc?*", false));
        assert!("LUNCH".matches_pattern("lunc?*", false));
        assert!(!" lunch".matches_pattern("lunc?*", false));
        assert!(!"lunc".matches_pattern("lunc?*", false));
    }

    fn sender() -> OwnedUserId {
        owned_user_id!("@worthy_whale:server.name")
    }

    fn push_context() -> PushConditionRoomCtx {
        let mut users = BTreeMap::new();
        users.insert(sender(), int!(25));

        let power_levels = PushConditionPowerLevelsCtx {
            users,
            users_default: int!(50),
            notifications: NotificationPowerLevels { room: int!(50) },
        };

        PushConditionRoomCtx {
            room_id: owned_room_id!("!room:server.name"),
            member_count: uint!(3),
            user_id: owned_user_id!("@gorilla:server.name"),
            user_display_name: "Groovy Gorilla".into(),
            power_levels: Some(power_levels),
            #[cfg(feature = "unstable-msc3931")]
            supported_features: Default::default(),
        }
    }

    fn first_flattened_event() -> FlattenedJson {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@worthy_whale:server.name",
                "content": {
                    "msgtype": "m.text",
                    "body": "@room Give a warm welcome to Groovy Gorilla"
                }
            }"#,
        )
        .unwrap();

        FlattenedJson::from_raw(&raw)
    }

    fn second_flattened_event() -> FlattenedJson {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@party_bot:server.name",
                "content": {
                    "msgtype": "m.notice",
                    "body": "Everybody come to party!"
                }
            }"#,
        )
        .unwrap();

        FlattenedJson::from_raw(&raw)
    }

    #[test]
    fn event_match_applies() {
        let context = push_context();
        let first_event = first_flattened_event();
        let second_event = second_flattened_event();

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
    }

    #[test]
    fn room_member_count_is_applies() {
        let context = push_context();
        let event = first_flattened_event();

        let member_count_eq =
            PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(3)) };
        let member_count_gt =
            PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(2)..) };
        let member_count_lt =
            PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(..uint!(3)) };

        assert!(member_count_eq.applies(&event, &context));
        assert!(member_count_gt.applies(&event, &context));
        assert!(!member_count_lt.applies(&event, &context));
    }

    #[test]
    fn contains_display_name_applies() {
        let context = push_context();
        let first_event = first_flattened_event();
        let second_event = second_flattened_event();

        let contains_display_name = PushCondition::ContainsDisplayName;

        assert!(contains_display_name.applies(&first_event, &context));
        assert!(!contains_display_name.applies(&second_event, &context));
    }

    #[test]
    fn sender_notification_permission_applies() {
        let context = push_context();
        let first_event = first_flattened_event();
        let second_event = second_flattened_event();

        let sender_notification_permission =
            PushCondition::SenderNotificationPermission { key: "room".into() };

        assert!(!sender_notification_permission.applies(&first_event, &context));
        assert!(sender_notification_permission.applies(&second_event, &context));
    }

    #[cfg(feature = "unstable-msc3932")]
    #[test]
    fn room_version_supports_applies() {
        let context_not_matching = push_context();

        let context_matching = PushConditionRoomCtx {
            room_id: owned_room_id!("!room:server.name"),
            member_count: uint!(3),
            user_id: owned_user_id!("@gorilla:server.name"),
            user_display_name: "Groovy Gorilla".into(),
            power_levels: context_not_matching.power_levels.clone(),
            supported_features: vec![super::RoomVersionFeature::ExtensibleEvents],
        };

        let simple_event_raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@worthy_whale:server.name",
                "content": {
                    "msgtype": "org.matrix.msc3932.extensible_events",
                    "body": "@room Give a warm welcome to Groovy Gorilla"
                }
            }"#,
        )
        .unwrap();
        let simple_event = FlattenedJson::from_raw(&simple_event_raw);

        let room_version_condition = PushCondition::RoomVersionSupports {
            feature: super::RoomVersionFeature::ExtensibleEvents,
        };

        assert!(room_version_condition.applies(&simple_event, &context_matching));
        assert!(!room_version_condition.applies(&simple_event, &context_not_matching));
    }

    #[test]
    fn event_property_is_applies() {
        use crate::push::condition::ScalarJsonValue;

        let context = push_context();
        let event_raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@worthy_whale:server.name",
                "content": {
                    "msgtype": "m.text",
                    "body": "Boom!",
                    "org.fake.boolean": false,
                    "org.fake.number": 13,
                    "org.fake.null": null
                }
            }"#,
        )
        .unwrap();
        let event = FlattenedJson::from_raw(&event_raw);

        let string_match = PushCondition::EventPropertyIs {
            key: "content.body".to_owned(),
            value: "Boom!".into(),
        };
        assert!(string_match.applies(&event, &context));

        let string_no_match =
            PushCondition::EventPropertyIs { key: "content.body".to_owned(), value: "Boom".into() };
        assert!(!string_no_match.applies(&event, &context));

        let wrong_type =
            PushCondition::EventPropertyIs { key: "content.body".to_owned(), value: false.into() };
        assert!(!wrong_type.applies(&event, &context));

        let bool_match = PushCondition::EventPropertyIs {
            key: r"content.org\.fake\.boolean".to_owned(),
            value: false.into(),
        };
        assert!(bool_match.applies(&event, &context));

        let bool_no_match = PushCondition::EventPropertyIs {
            key: r"content.org\.fake\.boolean".to_owned(),
            value: true.into(),
        };
        assert!(!bool_no_match.applies(&event, &context));

        let int_match = PushCondition::EventPropertyIs {
            key: r"content.org\.fake\.number".to_owned(),
            value: int!(13).into(),
        };
        assert!(int_match.applies(&event, &context));

        let int_no_match = PushCondition::EventPropertyIs {
            key: r"content.org\.fake\.number".to_owned(),
            value: int!(130).into(),
        };
        assert!(!int_no_match.applies(&event, &context));

        let null_match = PushCondition::EventPropertyIs {
            key: r"content.org\.fake\.null".to_owned(),
            value: ScalarJsonValue::Null,
        };
        assert!(null_match.applies(&event, &context));
    }

    #[test]
    fn event_property_contains_applies() {
        use crate::push::condition::ScalarJsonValue;

        let context = push_context();
        let event_raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@worthy_whale:server.name",
                "content": {
                    "org.fake.array": ["Boom!", false, 13, null]
                }
            }"#,
        )
        .unwrap();
        let event = FlattenedJson::from_raw(&event_raw);

        let wrong_key =
            PushCondition::EventPropertyContains { key: "send".to_owned(), value: false.into() };
        assert!(!wrong_key.applies(&event, &context));

        let string_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: "Boom!".into(),
        };
        assert!(string_match.applies(&event, &context));

        let string_no_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: "Boom".into(),
        };
        assert!(!string_no_match.applies(&event, &context));

        let bool_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: false.into(),
        };
        assert!(bool_match.applies(&event, &context));

        let bool_no_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: true.into(),
        };
        assert!(!bool_no_match.applies(&event, &context));

        let int_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: int!(13).into(),
        };
        assert!(int_match.applies(&event, &context));

        let int_no_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: int!(130).into(),
        };
        assert!(!int_no_match.applies(&event, &context));

        let null_match = PushCondition::EventPropertyContains {
            key: r"content.org\.fake\.array".to_owned(),
            value: ScalarJsonValue::Null,
        };
        assert!(null_match.applies(&event, &context));
    }
}
