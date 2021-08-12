//! Types for the *m.notifications_profile.<profile>* event.

use std::collections::BTreeMap;

use ruma_events_macros::EventContent;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::GlobalAccountDataEvent;

/// An event to store a "notifications profile" definition in a user's `account_data`.
pub type SomeProfileEvent = GlobalAccountDataEvent<SomeProfileEventContent>;

/// The payload for `SomeProfileEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.notifications_profile.<profile>", kind = GlobalAccountData)]
pub struct SomeProfileEventContent {
    /// A map from actions to the notification attributes which trigger the actions.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub actions: BTreeMap<NotificationAction, Vec<RequiredNotificationAttribute>>,
}

impl SomeProfileEventContent {
    /// Creates a new `SomeProfileEventContent` with the given actions.
    pub fn new(actions: BTreeMap<NotificationAction, Vec<RequiredNotificationAttribute>>) -> Self {
        Self { actions }
    }
}

/// Event notification actions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NotificationAction {
    /// Show a "native notification".
    #[ruma_enum(rename = "m.notify")]
    Notify,

    /// Play an audible alert.
    #[ruma_enum(rename = "m.sound")]
    Sound,

    /// Highlight the room containing the event.
    #[ruma_enum(rename = "m.highlight")]
    Highlight,

    #[doc(hidden)]
    _Custom(String),
}

/// A notification attribute or series of notification attributes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum RequiredNotificationAttribute {
    /// A single notification attribute.
    Single(NotificationAttribute),

    /// A sequence of notification attributes which must all be satisfied.
    Sequence(Vec<NotificationAttribute>),
}

/// Event notification attributes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NotificationAttribute {
    /// The event contains one of the user's registered "notification keywords".
    #[ruma_enum(rename = "m.keyword")]
    Keyword,

    /// The event contains a "mention" of the user's userid, etc.
    #[ruma_enum(rename = "m.mention")]
    Mention,

    /// The event is an invitation to a room.
    #[ruma_enum(rename = "m.invite")]
    Invite,

    /// The event is a notification that a room has been upgraded.
    #[ruma_enum(rename = "m.room_upgrade")]
    RoomUpgrade,

    /// The event is an invitation to a VoIP call.
    #[ruma_enum(rename = "m.voip_call")]
    VoipCall,

    /// The event was in a Direct Message room.
    #[ruma_enum(rename = "m.dm")]
    DirectMessage,

    /// The event contains a visible body.
    #[ruma_enum(rename = "m.msg")]
    VisibleBody,

    #[doc(hidden)]
    _Custom(String),
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use std::collections::BTreeMap;

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::notifications_profile::some_profile::{
        NotificationAction, NotificationAttribute, RequiredNotificationAttribute,
        SomeProfileEventContent,
    };

    #[test]
    fn deserialize_empty_profile_event() {
        let json = json!({});

        assert_matches!(
            from_json_value(json).unwrap(),
            SomeProfileEventContent { actions }
            if actions.is_empty()
        )
    }

    #[test]
    fn deserialize_some_profile_event() {
        let json = json!(
            {
                "actions": {
                  "m.notify": [
                      "m.mention", "m.keyword", "m.invite", "m.room_upgrade",
                      "m.voip_call", ["m.dm", "m.msg"]
                  ],
                  "m.sound": [
                      "m.mention", "m.keyword", "m.invite", "m.room_upgrade",
                      "m.voip_call", ["m.dm", "m.msg"]
                  ],
                  "m.highlight": ["m.mention", "m.keyword"]
                }
              }
        );

        let notify_and_sound_attributes = vec![
            RequiredNotificationAttribute::Single(NotificationAttribute::Mention),
            RequiredNotificationAttribute::Single(NotificationAttribute::Keyword),
            RequiredNotificationAttribute::Single(NotificationAttribute::Invite),
            RequiredNotificationAttribute::Single(NotificationAttribute::RoomUpgrade),
            RequiredNotificationAttribute::Single(NotificationAttribute::VoipCall),
            RequiredNotificationAttribute::Sequence(vec![
                NotificationAttribute::DirectMessage,
                NotificationAttribute::VisibleBody,
            ]),
        ];

        let highlight_attributes = vec![
            RequiredNotificationAttribute::Single(NotificationAttribute::Mention),
            RequiredNotificationAttribute::Single(NotificationAttribute::Keyword),
        ];

        let mut expected_actions = BTreeMap::new();
        expected_actions.insert(NotificationAction::Notify, notify_and_sound_attributes.clone());
        expected_actions.insert(NotificationAction::Sound, notify_and_sound_attributes);
        expected_actions.insert(NotificationAction::Highlight, highlight_attributes);

        assert_matches!(
            from_json_value(json).unwrap(),
            SomeProfileEventContent {
                actions
            }
            if actions == expected_actions
        );
    }

    #[test]
    fn serialize_empty_profile_event() {
        let json = json!({});

        let content = SomeProfileEventContent::new(BTreeMap::new());

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn serialize_some_profile_event() {
        let json = json!(
            {
                "actions": {
                  "m.notify": [
                      "m.mention", "m.keyword", "m.invite", "m.room_upgrade",
                      "m.voip_call", ["m.dm", "m.msg"]
                  ],
                  "m.sound": [
                      "m.mention", "m.keyword", "m.invite", "m.room_upgrade",
                      "m.voip_call", ["m.dm", "m.msg"]
                  ],
                  "m.highlight": ["m.mention", "m.keyword"]
                }
              }
        );

        let notify_and_sound_attributes = vec![
            RequiredNotificationAttribute::Single(NotificationAttribute::Mention),
            RequiredNotificationAttribute::Single(NotificationAttribute::Keyword),
            RequiredNotificationAttribute::Single(NotificationAttribute::Invite),
            RequiredNotificationAttribute::Single(NotificationAttribute::RoomUpgrade),
            RequiredNotificationAttribute::Single(NotificationAttribute::VoipCall),
            RequiredNotificationAttribute::Sequence(vec![
                NotificationAttribute::DirectMessage,
                NotificationAttribute::VisibleBody,
            ]),
        ];

        let highlight_attributes = vec![
            RequiredNotificationAttribute::Single(NotificationAttribute::Mention),
            RequiredNotificationAttribute::Single(NotificationAttribute::Keyword),
        ];

        let mut actions = BTreeMap::new();
        actions.insert(NotificationAction::Notify, notify_and_sound_attributes.clone());
        actions.insert(NotificationAction::Sound, notify_and_sound_attributes);
        actions.insert(NotificationAction::Highlight, highlight_attributes);

        let content = SomeProfileEventContent::new(actions);

        assert_eq!(to_json_value(&content).unwrap(), json);
    }
}
