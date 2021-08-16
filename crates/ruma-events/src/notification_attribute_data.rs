//! Types for the *m.notification_attribute_data* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::GlobalAccountDataEvent;

/// An event to store assignment of event notification attributes in a user's `account_data`.
pub type NotificationAttributeDataEvent =
    GlobalAccountDataEvent<NotificationAttributeDataEventContent>;

/// The payload for `NotificationAttributeDataEvent`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc2785.notification_attribute_data", kind = GlobalAccountData)]
pub struct NotificationAttributeDataEventContent {
    /// An array of string which form "notification keywords".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// An object containing booleans which define which events should qualify for `m.mention`
    /// attributes.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub mentions: Mentions,
}

impl NotificationAttributeDataEventContent {
    /// Creates a new `NotificationAttributeDataEventContent with the given keywords and mentions.
    pub fn new(keywords: Vec<String>, mentions: Mentions) -> Self {
        Self { keywords, mentions }
    }
}

/// An object containing booleans which define which events should qualify for `m.mention`
/// attributes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Mentions {
    /// Display name flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub displayname: bool,

    /// Matrix user ID flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub mxid: bool,

    /// Local part of user ID flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub localpart: bool,

    /// "@room" notification flag.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub room_notif: bool,
}

impl Mentions {
    /// Creates a new `Mentions` with the specified flags.
    pub fn new(displayname: bool, mxid: bool, localpart: bool, room_notif: bool) -> Self {
        Self { displayname, mxid, localpart, room_notif }
    }
}

#[cfg(test)]
mod tests {

    use matches::assert_matches;

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::notification_attribute_data::{Mentions, NotificationAttributeDataEventContent};

    #[test]
    fn test_empty_notification_attribute_data_serialization() {
        let json = json!({});

        let content = NotificationAttributeDataEventContent::default();

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn test_notification_attribute_data_serialization() {
        let json = json!(
            {
            "keywords": ["foo", "bar", "longer string"],
            "mentions": {
               "displayname": true,
               "mxid": true,
               "localpart": true,
               "room_notif": true
            }
          }
        );

        let content = NotificationAttributeDataEventContent::new(
            vec!["foo".into(), "bar".into(), "longer string".into()],
            Mentions::new(true, true, true, true),
        );

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn test_empty_notification_attribute_data_deserialization() {
        let json = json!({});

        assert_matches!(
            from_json_value(json).unwrap(),
            NotificationAttributeDataEventContent {
                keywords,
                mentions
            }
            if keywords.is_empty()
                    && ruma_serde::is_default(&mentions)
        )
    }

    #[test]
    fn test_notification_attribute_data_deserialization() {
        let json = json!(
            {
            "keywords": ["foo", "bar", "longer string"],
            "mentions": {
               "displayname": true,
               "mxid": true,
               "localpart": true,
               "room_notif": true
            }
          }
        );

        let expected_keywords: Vec<String> =
            vec!["foo".into(), "bar".into(), "longer string".into()];
        assert_matches!(
            from_json_value(json).unwrap(),
            NotificationAttributeDataEventContent { keywords, mentions: Mentions {
                displayname: true,
                mxid: true,
                localpart: true,
                room_notif: true,
            } }
            if keywords == expected_keywords
        );
    }
}
