use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue as RawJsonValue;

use super::PushCondition;
use crate::{
    push::_CustomPushCondition,
    serde::{JsonObject, from_raw_json_value},
};

impl<'de> Deserialize<'de> for PushCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ConditionDeHelper {
            kind: String,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let ConditionDeHelper { kind } = from_raw_json_value(&json)?;

        match kind.as_ref() {
            "event_match" => from_raw_json_value(&json).map(Self::EventMatch),
            #[allow(deprecated)]
            "contains_display_name" => Ok(Self::ContainsDisplayName),
            "room_member_count" => from_raw_json_value(&json).map(Self::RoomMemberCount),
            "sender_notification_permission" => {
                from_raw_json_value(&json).map(Self::SenderNotificationPermission)
            }
            #[cfg(feature = "unstable-msc3931")]
            "org.matrix.msc3931.room_version_supports" => {
                from_raw_json_value(&json).map(Self::RoomVersionSupports)
            }
            "event_property_is" => from_raw_json_value(&json).map(Self::EventPropertyIs),
            "event_property_contains" => {
                from_raw_json_value(&json).map(Self::EventPropertyContains)
            }
            #[cfg(feature = "unstable-msc4306")]
            "io.element.msc4306.thread_subscription" => {
                from_raw_json_value(&json).map(Self::ThreadSubscription)
            }
            _ => {
                let mut data = from_raw_json_value::<JsonObject, _>(&json)?;
                data.remove("kind");

                Ok(Self::_Custom(_CustomPushCondition { kind, data }))
            }
        }
    }
}
