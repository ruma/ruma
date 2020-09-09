//! [POST /_matrix/push/v1/notify](https://matrix.org/docs/spec/push_gateway/r0.1.1#post-matrix-push-v1-notify)

use js_int::UInt;
use ruma_api::{ruma_api, Outgoing};
pub use ruma_common::push::{PusherData, Tweak};
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;
use std::time::SystemTime;
use strum::{Display, EnumString};

ruma_api! {
    metadata: {
        description: "Notify a push gateway about an event or update the number of unread notifications a user has",
        name: "send_event_notification",
        method: POST,
        path: "/_matrix/push/v1/notify",
        rate_limited: false,
        requires_authentication: false,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    request: {
        /// Information about the push notification
        pub notification: Notification<'a>,
    }

    #[derive(Default)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    response: {
        /// A list of all pushkeys given in the notification request that are
        /// not valid.
        ///
        /// These could have been rejected by an upstream gateway because they
        /// have expired or have never been valid. Homeservers must cease
        /// sending notification requests for these pushkeys and remove the
        /// associated pushers. It may not necessarily be the notification in
        /// the request that failed: it could be that a previous notification to
        /// the same pushkey failed. May be empty.
        pub rejected: Vec<String>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given notification.
    pub fn new(notification: Notification<'a>) -> Self {
        Self { notification }
    }
}

impl Response {
    /// Creates a new `Response` with the given list of rejected pushkeys.
    pub fn new(rejected: Vec<String>) -> Self {
        Self { rejected }
    }
}

/// Type for passing information about a push notification
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Notification<'a> {
    /// The Matrix event ID of the event being notified about.
    ///
    /// This is required if the notification is about a particular Matrix event.
    /// It may be omitted for notifications that only contain updated badge
    /// counts. This ID can and should be used to detect duplicate notification
    /// requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<&'a EventId>,

    /// The ID of the room in which this event occurred.
    ///
    /// Required if the notification relates to a specific Matrix event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<&'a RoomId>,

    /// The type of the event as in the event's `type` field.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<&'a EventType>,

    /// The sender of the event as in the corresponding event field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<&'a UserId>,

    /// The current display name of the sender in the room in which the event
    /// occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_display_name: Option<&'a str>,

    /// The name of the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<&'a str>,

    /// An alias to display for the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_alias: Option<&'a RoomAliasId>,

    /// This is `true` if the user receiving the notification is the subject of
    /// a member event (i.e. the `state_key` of the member event is equal to the
    /// user's Matrix ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_is_target: Option<bool>,

    /// The priority of the notification.
    ///
    /// If omitted, `high` is assumed. This may be used by push gateways to
    /// deliver less time-sensitive notifications in a way that will preserve
    /// battery power on mobile devices. One of: ["high", "low"]
    #[serde(skip_serializing_if = "ruma_serde::is_default")]
    pub prio: NotificationPriority,

    /// The `content` field from the event, if present. The pusher may omit this
    /// if the event had no content or for any other reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Box<RawJsonValue>>,

    /// This is a dictionary of the current number of unacknowledged
    /// communications for the recipient user. Counts whose value is zero should
    /// be omitted.
    #[serde(skip_serializing_if = "ruma_serde::is_default")]
    pub counts: NotificationCounts,

    /// This is an array of devices that the notification should be sent to.
    pub devices: &'a [Device],
}

impl<'a> Notification<'a> {
    /// Create a new notification for the given devices
    pub fn new(devices: &'a [Device]) -> Self {
        Notification { devices, ..Default::default() }
    }
}

/// Type for passing information about notification priority.
///
/// This may be used by push gateways to deliver less time-sensitive
/// notifications in a way that will preserve battery power on mobile devices.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Display, EnumString, PartialEq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NotificationPriority {
    /// A high priority notification
    High,
    /// A low priority notification
    Low,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::High
    }
}

/// Type for passing information about notification counts.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NotificationCounts {
    /// The number of unread messages a user has across all of the rooms they
    /// are a member of.
    #[serde(skip_serializing_if = "ruma_serde::is_default")]
    pub unread: UInt,

    /// The number of unacknowledged missed calls a user has across all rooms of
    /// which they are a member.
    #[serde(skip_serializing_if = "ruma_serde::is_default")]
    pub missed_calls: UInt,
}

impl NotificationCounts {
    /// Create new notification counts from the given unread and missed call
    /// counts.
    pub fn new(unread: UInt, missed_calls: UInt) -> Self {
        NotificationCounts { unread, missed_calls }
    }
}

/// Type for passing information about devices.
#[derive(Clone, Debug, Deserialize, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Device {
    /// The `app_id` given when the pusher was created.
    ///
    /// Max length, 64 chars.
    pub app_id: String,

    /// The `pushkey` given when the pusher was created.
    ///
    /// Max length, 512 bytes.
    pub pushkey: String,

    /// The unix timestamp (in seconds) when the pushkey was last updated.
    #[serde(
        with = "ruma_serde::time::opt_ms_since_unix_epoch",
        skip_serializing_if = "Option::is_none"
    )]
    pub pushkey_ts: Option<SystemTime>,

    /// A dictionary of additional pusher-specific data. For 'http' pushers,
    /// this is the data dictionary passed in at pusher creation minus the `url`
    /// key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<PusherData>,

    /// A dictionary of customisations made to the way this notification is to
    /// be presented. These are added by push rules.
    #[serde(with = "tweak", skip_serializing_if = "Vec::is_empty")]
    pub tweaks: Vec<Tweak>,
}

impl Device {
    /// Create a new device with:
    /// * the given app id
    /// * pushkey given when the pusher was created
    /// * the timestamp when the pushkey was last updated
    /// * additional pusher data which must contain a `url` key.
    /// * customisations made to the way notifications are presented.
    pub fn new(app_id: String, pushkey: String) -> Self {
        Device { app_id, pushkey, pushkey_ts: None, data: None, tweaks: Vec::new() }
    }
}

mod tweak {
    use std::fmt;

    use ruma_common::push::Tweak;
    use serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserializer, Serializer,
    };

    pub fn serialize<S>(tweak: &[super::Tweak], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(tweak.len()))?;
        for item in tweak {
            #[allow(unreachable_patterns)]
            match item {
                Tweak::Highlight(b) => map.serialize_entry("highlight", b)?,
                Tweak::Sound(value) => map.serialize_entry("sound", value)?,
                Tweak::Custom { value, name } => map.serialize_entry(name, value)?,
                _ => unreachable!("variant added to Tweak not covered by Custom"),
            }
        }
        map.end()
    }

    struct TweaksVisitor;

    impl<'de> Visitor<'de> for TweaksVisitor {
        type Value = Vec<Tweak>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Lazy load options")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut tweaks = vec![];
            while let Some(key) = access.next_key::<String>()? {
                match &*key {
                    "sound" => tweaks.push(Tweak::Sound(access.next_value()?)),
                    // If a highlight tweak is given with no value, its value is defined to be true.
                    "highlight" => {
                        let highlight =
                            if let Ok(highlight) = access.next_value() { highlight } else { true };

                        tweaks.push(Tweak::Highlight(highlight))
                    }
                    // TODO should this be an error?
                    _ => tweaks.push(Tweak::Custom { name: key, value: access.next_value()? }),
                };
            }

            // If no highlight tweak is given at all then the value of highlight is defined to be false.
            if !tweaks.iter().any(|tw| matches!(tw, Tweak::Highlight(_))) {
                tweaks.push(Tweak::Highlight(false));
            }

            Ok(tweaks)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Tweak>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TweaksVisitor)
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use ruma_events::EventType;
    use ruma_identifiers::{event_id, room_alias_id, room_id, user_id};
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use super::{Device, Notification, NotificationCounts, NotificationPriority, Tweak};

    #[test]
    fn serialize_request() {
        let expected = json!({
            "event_id": "$3957tyerfgewrf384",
            "room_id": "!slw48wfj34rtnrf:example.com",
            "type": "m.room.message",
            "sender": "@exampleuser:matrix.org",
            "sender_display_name": "Major Tom",
            "room_alias": "#exampleroom:matrix.org",
            "prio": "low",
            "content": {},
            "counts": {
              "unread": 2,
            },
            "devices": [
              {
                "app_id": "org.matrix.matrixConsole.ios",
                "pushkey": "V2h5IG9uIGVhcnRoIGRpZCB5b3UgZGVjb2RlIHRoaXM/",
                "pushkey_ts": 123,
                "tweaks": {
                  "sound": "silence",
                  "highlight": true,
                  "custom": "go wild"
                }
              }
            ]
        });

        let eid = event_id!("$3957tyerfgewrf384");
        let rid = room_id!("!slw48wfj34rtnrf:example.com");
        let uid = user_id!("@exampleuser:matrix.org");
        let alias = room_alias_id!("#exampleroom:matrix.org");

        let mut count = NotificationCounts::default();
        count.unread = js_int::uint!(2);
        // test default values are ignored
        count.missed_calls = js_int::uint!(0);

        let mut device = Device::new(
            "org.matrix.matrixConsole.ios".into(),
            "V2h5IG9uIGVhcnRoIGRpZCB5b3UgZGVjb2RlIHRoaXM/".into(),
        );
        device.pushkey_ts = Some(SystemTime::UNIX_EPOCH + Duration::from_millis(123));
        device.tweaks = vec![
            Tweak::Highlight(true),
            Tweak::Sound("silence".into()),
            Tweak::Custom {
                name: "custom".into(),
                value: from_json_value(JsonValue::String("go wild".into())).unwrap(),
            },
        ];

        let devices = vec![device];

        let mut notice = Notification::default();
        notice.event_id = Some(&eid);
        notice.room_id = Some(&rid);
        notice.event_type = Some(&EventType::RoomMessage);
        notice.sender = Some(&uid);
        notice.sender_display_name = Some("Major Tom");
        notice.room_alias = Some(&alias);
        notice.content = Some(serde_json::from_str("{}").unwrap());
        notice.counts = count;
        notice.prio = NotificationPriority::Low;
        notice.devices = &devices;

        assert_eq!(expected, to_json_value(notice).unwrap())
    }
}
