use serde::{de, Deserialize, Serialize, Serializer};
use serde_json::value::RawValue as RawJsonValue;

use crate::serde::from_raw_json_value;

#[cfg(feature = "unstable-msc3931")]
use super::RoomVersionFeature;
use super::{PushCondition, RoomMemberCountIs};

impl Serialize for PushCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PushCondition::_Custom(custom) => custom.serialize(serializer),
            _ => PushConditionSerDeHelper::from(self.clone()).serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PushCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let ExtractKind { kind } = from_raw_json_value(&json)?;

        match kind.as_ref() {
            "event_match"
            | "contains_display_name"
            | "room_member_count"
            | "sender_notification_permission" => {
                let helper: PushConditionSerDeHelper = from_raw_json_value(&json)?;
                Ok(helper.into())
            }
            #[cfg(feature = "unstable-msc3931")]
            "org.matrix.msc3931.room_version_supports" => {
                let helper: PushConditionSerDeHelper = from_raw_json_value(&json)?;
                Ok(helper.into())
            }
            _ => from_raw_json_value(&json).map(Self::_Custom),
        }
    }
}

#[derive(Deserialize)]
struct ExtractKind {
    kind: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PushConditionSerDeHelper {
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

    /// Apply the rule only to rooms that support a given feature.
    #[cfg(feature = "unstable-msc3931")]
    #[serde(rename = "org.matrix.msc3931.room_version_supports")]
    RoomVersionSupports {
        /// The feature the room must support for the push rule to apply.
        feature: RoomVersionFeature,
    },
}

impl From<PushConditionSerDeHelper> for PushCondition {
    fn from(value: PushConditionSerDeHelper) -> Self {
        match value {
            PushConditionSerDeHelper::EventMatch { key, pattern } => {
                Self::EventMatch { key, pattern }
            }
            PushConditionSerDeHelper::ContainsDisplayName => Self::ContainsDisplayName,
            PushConditionSerDeHelper::RoomMemberCount { is } => Self::RoomMemberCount { is },
            PushConditionSerDeHelper::SenderNotificationPermission { key } => {
                Self::SenderNotificationPermission { key }
            }
            #[cfg(feature = "unstable-msc3931")]
            PushConditionSerDeHelper::RoomVersionSupports { feature } => {
                Self::RoomVersionSupports { feature }
            }
        }
    }
}

impl From<PushCondition> for PushConditionSerDeHelper {
    fn from(value: PushCondition) -> Self {
        match value {
            PushCondition::EventMatch { key, pattern } => Self::EventMatch { key, pattern },
            PushCondition::ContainsDisplayName => Self::ContainsDisplayName,
            PushCondition::RoomMemberCount { is } => Self::RoomMemberCount { is },
            PushCondition::SenderNotificationPermission { key } => {
                Self::SenderNotificationPermission { key }
            }
            #[cfg(feature = "unstable-msc3931")]
            PushCondition::RoomVersionSupports { feature } => Self::RoomVersionSupports { feature },
            PushCondition::_Custom(_) => unimplemented!(),
        }
    }
}
